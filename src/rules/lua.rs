use lua;
use rules::Rule;
use std::path::Path;
use std::collections::HashMap;
use std::rc::Rc;
use ::emu::emu_engine::EmuEffects;

const LUARULES_REG_KEY: &'static str = "dirt";

#[derive(Debug)]
pub enum Error {
    LuaError(String),
    NotImplemented,
}

pub struct LuaRules {
    lua: ::lua::State,
    candidates_rules: HashMap<String, Vec<Rule>>,
}

fn lua_rule(lua: &mut ::lua::State) -> i32 {
    lua.push_string(LUARULES_REG_KEY);
    lua.get_table(lua::REGISTRYINDEX);
    let lua_rule = lua.to_integer(-1);

    let name = lua.to_str(1)
        .unwrap()
        .to_owned();
    let top = lua.get_top();
    let mut args = Vec::new();
    for i in 2..top - 1 {
        let arg = lua.to_str(i).unwrap().to_owned();
        args.push(arg);
    }

    println!("[{:x}] New rule: {:?}, {:?}", lua_rule, name, args);
    return 0;
}

impl LuaRules {
    pub fn new() -> LuaRules {
        let mut state = lua::State::new();
        state.push_fn(lua_func!(lua_rule));
        state.set_global("rule");

        state.push_string(LUARULES_REG_KEY);
        state.push_integer(0xdeadbeef);
        state.set_table(lua::REGISTRYINDEX);

        let mut candidates_rules = HashMap::new();
        let args = ::emu::args::EmuArgs::new(vec![
               Rc::new(::emu::datatypes::StringData::new("AAAAAAAAAA")),
               Rc::new(::emu::datatypes::StringData::new("AA %s CC")),
               Rc::new(::emu::datatypes::StringData::new("BB")),
            ]);

        let verifier = Box::new(|r: &EmuEffects| {
            let pushed_args = r.args.pushed_args();
            if let Ok(a) = r.vmstate.read_str(pushed_args[0]) {
                return r.return_value as i32 == 8 && a == "AA BB CC";
            } else {
                return false;
            }
        });
        candidates_rules.insert(String::from("sprintf"),
                                vec![Rule {
                                         name: String::from("sprintf"),
                                         args: args,
                                         verifier: verifier,
                                     }]);
        return LuaRules {
            lua: state,
            candidates_rules: candidates_rules,
        };
    }

    pub fn load(&mut self, path: &Path) -> Result<(), Error> {
        println!("{:?}", path);
        let r = self.lua.load_file(path.to_str().unwrap());
        if r.is_err() {
            return Err(self.pop_error());
        }

        let r = self.lua.pcall(0, 0, 0);
        if r.is_err() {
            return Err(self.pop_error());
        }

        return Ok(());
    }

    pub fn candidates(&self) -> &HashMap<String, Vec<Rule>> {
        return &self.candidates_rules;
    }

    fn pop_error(&mut self) -> Error {
        let err = Error::LuaError(self.lua.to_str(-1).unwrap().to_owned());
        self.lua.pop(1);
        return err;
    }
}
