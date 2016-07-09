use emu::args::EmuArgs;
use emu::datatypes::{DataType, StringData};
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

pub struct LuaEffects<'s> {
    pub effects: &'s EmuEffects<'s>,
}

fn lua_effect_return_value(lua: &mut ::lua::State) -> i32 {
    let value = {
        let effects: &mut Option<&EmuEffects> =
            unsafe { lua.to_userdata_typed(1) }.unwrap();

        effects
            .expect("EmuEffects should not be used outside of test validation")
            .return_value
    };
    lua.push_integer(value as i64);
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
            let dirt_fns = &[("rule", lua_func!(lua_rule))];
            lua.new_lib(dirt_fns);
            lua.set_global("Dirt");

            let effects_fns = &[("return_value",
                                 lua_func!(lua_effect_return_value))];
            lua.new_metatable("EmuEffects");
            lua.new_lib_table(effects_fns);
            lua.set_fns(effects_fns, 0);
            lua.set_field(-2, "__index");

            lua.set_global("return_value");

            lua.load_library(::lua::Library::Base);
            lua.load_library(::lua::Library::Io);
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
            let arg = lua.to_str(i).unwrap().to_owned();
            args.push(Rc::new(StringData::new(&arg)));
            lua.pop(1);
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
