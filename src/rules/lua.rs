use emu::args::EmuArgs;
use emu::datatypes::{BufData, DataType, IntegerData, StringData};
use emu::emu_engine::EmuEffects;
use lua;
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
        Err(_) => lua.push_string(""),
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

        if r.is_err() {
            panic!("{:?}", pop_error(&mut lua));
        }

        *lua_effects = None;
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

struct LuaBufData(u64);

fn lua_buf(lua: &mut ::lua::State) -> i32 {
    let size = lua.to_integer(-1);
    let buf: *mut LuaBufData = lua.new_userdata_typed();
    if buf.is_null() {
        panic!("Lua error");
    }
    lua.set_metatable_from_registry("BufData");

    (unsafe { &mut *buf }).0 = size as u64;
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
                             ("Buf", lua_func!(lua_buf))];
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
        return &self.candidates_rules;
    }

    fn on_rule(&mut self, lua: &mut ::lua::State) -> i32 {
        let name = lua.to_str(1)
            .unwrap()
            .to_owned();
        lua.pop(1);
        let top = lua.get_top();
        let mut args: Vec<Rc<DataType>> = Vec::new();
        for i in 2..top {
            if lua.is_integer(i) {
                let arg = lua.to_integer(i) as u64;
                args.push(Rc::new(IntegerData(arg)));
            } else if lua.is_string(i) {
                let arg = lua.to_str(i).unwrap().to_owned();
                args.push(Rc::new(StringData::new(&arg)));
                lua.pop(1);
            } else if let Some(&mut LuaBufData(size)) = unsafe {
                lua.test_userdata_typed(i, "BufData")
            } {
                args.push(Rc::new(BufData::new(size)));
            } else {
                panic!("Unsupported type: {}", lua.typename_at(i));
            }
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
}
