use emu::args::EmuArgs;
use emu::datatypes::{BufData, ByteData, CompositeData, DataType, IntegerData,
                     StringData, ThisOffsetData};
use emu::emu_engine::EmuEffects;
use lua;
use std::env;
use std::path::Path;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub use rules::rule::Rule;

const LUARULES_REG_KEY: &'static str = "dirt";

#[derive(Debug)]
pub enum Error {
    LuaError(String),
    NotImplemented,
}

fn lua_effect_return_value(lua: &mut ::lua::State) -> i32 {
    let value = {
        let udata = lua.check_userdata(1, "EmuEffects");
        if udata.is_null() {
            panic!("First arg must be EmuEffects");
        }
        let effects: &mut Option<&EmuEffects> =
            &mut unsafe { *(udata as *mut Option<&EmuEffects>) };

        effects
            .expect("EmuEffects should not be used outside of test validation")
            .return_value
    };
    lua.push_integer(value as i64);
    return 1;
}

fn lua_effect(lua: &mut ::lua::State) -> &EmuEffects {
    let udata = lua.check_userdata(1, "EmuEffects");
    if udata.is_null() {
        panic!("First arg must be EmuEffects");
    }
    let effects: &mut Option<&EmuEffects> =
        &mut unsafe { *(udata as *mut Option<&EmuEffects>) };

    return effects
            .expect("EmuEffects should not be used outside of test validation");
}

fn lua_effect_arg(lua: &mut ::lua::State) -> i32 {
    let n = lua.to_integer(1);
    let value = lua_effect(lua).args.nth(n as usize);
    lua.push_integer(value as i64);
    return 1;
}

fn lua_effect_str(lua: &mut ::lua::State) -> i32 {
    let addr = lua.to_integer(2);
    match lua_effect(lua).vmstate.read_str(addr as u64) {
        Ok(s) => lua.push_string(&s),
        Err(_) => lua.push_nil(),
    }
    return 1;
}

fn lua_effect_usize(lua: &mut ::lua::State) -> i32 {
    let addr = lua.to_integer(2);
    match lua_effect(lua).vmstate.read_usize(addr as u64) {
        Ok(value) => lua.push_integer(value as i64),
        Err(_) => lua.push_nil(),
    }
    return 1;
}

pub struct LuaRule {
    pub lua: Weak<RefCell<::lua::State>>,
    pub fn_ref: ::lua::Reference,
    pub name: String,
    pub args: EmuArgs,
}

impl Rule for LuaRule {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn args(&self) -> &EmuArgs {
        return &self.args;
    }

    fn verify(&self, effects: &EmuEffects) -> bool {
        let lua_ref = self.lua.upgrade().unwrap();
        let mut lua = lua_ref.borrow_mut();
        lua.raw_geti(lua::REGISTRYINDEX, self.fn_ref.value() as i64);

        let lua_effects_ptr: *mut Option<&EmuEffects> =
            lua.new_userdata_typed();
        if lua_effects_ptr.is_null() {
            panic!("Fail to create LuaEffects");
        }

        lua.set_metatable_from_registry("EmuEffects");

        let lua_effects = unsafe { &mut *lua_effects_ptr };
        *lua_effects = Some(effects);
        let r = lua.pcall(1, 1, 0);
        *lua_effects = None;

        if r.is_err() {
            println!("{:?}", pop_error(&mut lua));
            return false;
        }

        return lua.to_bool(-1);
    }
}

pub struct LuaRules {
    lua: Rc<RefCell<::lua::State>>,
    candidates_rules: HashMap<String, Vec<LuaRule>>,
}

fn lua_rule(lua: &mut ::lua::State) -> i32 {
    lua.push_string(LUARULES_REG_KEY);
    lua.get_table(lua::REGISTRYINDEX);
    let lua_rules_udata = lua.to_userdata(-1) as *mut LuaRules;
    if lua_rules_udata.is_null() {
        panic!("Dirt userdata is null");
    }

    lua.pop(1);
    let lua_rules: &mut LuaRules = unsafe { &mut *lua_rules_udata };
    return lua_rules.on_rule(lua);
}

struct LuaBufData(u64, Option<String>);

fn lua_buf(lua: &mut ::lua::State) -> i32 {
    let size = lua.to_integer(1);
    let buf: *mut LuaBufData = lua.new_userdata_typed();
    if buf.is_null() {
        panic!("Lua error");
    }
    lua.set_metatable_from_registry("BufData");

    let data = if lua.is_string(2) {
        let string = lua.to_str(2).unwrap().to_owned();
        lua.pop(1);
        Some(string)
    } else {
        None
    };

    unsafe { ::std::ptr::write(buf, LuaBufData(size as u64, data)) };
    return 1;
}

fn lua_buf_gc(lua: &mut ::lua::State) -> i32 {
    let v = lua.check_userdata(1, "BufData") as *mut LuaBufData;
    unsafe { ::std::ptr::drop_in_place(v) };
    return 0;
}

struct LuaThisData(u64);

fn lua_this(lua: &mut ::lua::State) -> i32 {
    let offset = lua.to_integer(-1);
    let buf: *mut LuaThisData = lua.new_userdata_typed();
    if buf.is_null() {
        panic!("Lua error");
    }
    lua.set_metatable_from_registry("ThisData");

    unsafe { ::std::ptr::write(buf, LuaThisData(offset as u64)) };
    return 1;
}

struct LuaByteData(u8);

fn lua_byte(lua: &mut ::lua::State) -> i32 {
    let byte = if lua.is_string(1) {
        let s = lua.to_str(1).unwrap().to_owned();
        lua.pop(1);
        s.as_bytes()[0]
    } else {
        lua.to_integer(1) as u8
    };
    let buf: *mut LuaByteData = lua.new_userdata_typed();
    if buf.is_null() {
        panic!("Lua error");
    }
    lua.set_metatable_from_registry("ByteData");

    unsafe { ::std::ptr::write(buf, LuaByteData(byte)) };
    return 1;
}

fn pop_error(lua: &mut ::lua::State) -> Error {
    let err = Error::LuaError(lua.to_str(-1).unwrap().to_owned());
    lua.pop(1);
    return err;
}


impl LuaRules {
    pub fn new() -> Box<LuaRules> {
        let mut lua_rules = Box::new(LuaRules {
            lua: Rc::new(RefCell::new(lua::State::new())),
            candidates_rules: HashMap::new(),
        });

        // Interface all the helpers functions.
        {
            let mut lua = lua_rules.lua.borrow_mut();
            let dirt_fns = &[("rule", lua_func!(lua_rule)),
                             ("Buf", lua_func!(lua_buf)),
                             ("Byte", lua_func!(lua_byte)),
                             ("This", lua_func!(lua_this))];
            lua.new_lib(dirt_fns);
            lua.set_global("Dirt");

            let effects_fns = &[("return_value",
                                 lua_func!(lua_effect_return_value)),
                                ("arg", lua_func!(lua_effect_arg)),
                                ("str", lua_func!(lua_effect_str)),
                                ("usize", lua_func!(lua_effect_usize))];
            lua.new_metatable("EmuEffects");
            lua.new_lib_table(effects_fns);
            lua.set_fns(effects_fns, 0);
            lua.set_field(-2, "__index");

            lua.new_metatable("BufData");
            lua.push_fn(lua_func!(lua_buf_gc));
            lua.set_field(-2, "__gc");

            lua.new_metatable("ThisData");
            lua.new_metatable("ByteData");

            lua.load_library(::lua::Library::Base);
            lua.load_library(::lua::Library::Io);
            lua.load_library(::lua::Library::String);
        }

        // Register the LuaRules object to the lua internal registry for future
        // reference.
        let lua_rules_udata: *mut LuaRules = &mut *lua_rules as *mut LuaRules;
        {
            let mut lua = lua_rules.lua.borrow_mut();
            lua.push_string(LUARULES_REG_KEY);
            unsafe { lua.push_light_userdata(lua_rules_udata) };
            lua.set_table(lua::REGISTRYINDEX);
        }

        return lua_rules;
    }

    pub fn load(&mut self, path: &Path) -> Result<(), Error> {
        let mut lua = self.lua.borrow_mut();
        let r = lua.load_file(path.to_str().unwrap());
        if r.is_err() {
            return Err(pop_error(&mut lua));
        }

        let r = lua.pcall(0, 0, 0);
        if r.is_err() {
            return Err(pop_error(&mut lua));
        }

        return Ok(());
    }

    pub fn add_rule(&mut self, rule: LuaRule) {
        if self.candidates_rules.contains_key(&rule.name) {
            self.candidates_rules.get_mut(&rule.name).unwrap().push(rule);
        } else {
            self.candidates_rules.insert(rule.name.clone(), vec![rule]);
        }
    }

    pub fn candidates(&self) -> &HashMap<String, Vec<LuaRule>> {
        &self.candidates_rules
    }

    fn on_rule(&mut self, lua: &mut ::lua::State) -> i32 {
        let name = lua.to_str(1)
            .unwrap()
            .to_owned();
        if let Ok(filter) = env::var("FILTER") {
            if filter != name {
                return 0;
            }
        }

        lua.pop(1);
        let top = lua.get_top();
        let mut args: Vec<Rc<DataType>> = Vec::new();
        for arg_n in 2..top {
            args.push(self.parse_rule_argument(lua, arg_n));
        }
        let fn_ref = lua.reference(lua::REGISTRYINDEX);

        let rule: LuaRule = LuaRule {
            lua: Rc::downgrade(&self.lua),
            fn_ref: fn_ref,
            name: name,
            args: EmuArgs::new(args),
        };

        self.add_rule(rule);

        return 0;
    }

    fn parse_rule_argument(&mut self,
                           lua: &mut ::lua::State,
                           arg_n: i32)
                           -> Rc<DataType> {
        if lua.is_integer(arg_n) {
            let arg = lua.to_integer(arg_n) as u64;
            return Rc::new(IntegerData(arg));
        }
        if lua.is_string(arg_n) {
            let arg = lua.to_str(arg_n).unwrap().to_owned();
            lua.pop(1);
            return Rc::new(StringData::new(&arg));
        }
        if lua.is_table(arg_n) {
            // Iterate on the table elements.
            let mut table_args = Vec::new();
            lua.push_nil();
            while lua.next(arg_n) {
                table_args.push(self.parse_rule_argument(lua, -1));
                lua.pop(1);
            }
            return Rc::new(CompositeData::new(table_args));
        }
        {
            if let Some(&mut LuaBufData(size, ref data)) = unsafe {
                lua.test_userdata_typed(arg_n, "BufData")
            } {
                return Rc::new(BufData::new(size,
                                            data.clone().map(|s| s.into())));
            }
        }
        {
            if let Some(&mut LuaThisData(offset)) = unsafe {
                lua.test_userdata_typed(arg_n, "ThisData")
            } {
                return Rc::new(ThisOffsetData(offset));
            }
        }
        {
            if let Some(&mut LuaByteData(byte)) = unsafe {
                lua.test_userdata_typed(arg_n, "ByteData")
            } {
                return Rc::new(ByteData(byte));
            }
        }

        panic!("Unsupported type: {}", lua.typename_at(arg_n));
    }
}
