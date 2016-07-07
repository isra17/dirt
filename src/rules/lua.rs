use lua;
use rules::Rule;
use std::path::Path;
use std::collections::HashMap;
use std::rc::Rc;
use ::emu::emu_engine::EmuEffects;

#[derive(Debug)]
pub enum Error {
    NotImplemented,
}

pub struct LuaRules {
    lua: ::lua::State,
    candidates_rules: HashMap<String, Vec<Rule>>,
}

impl LuaRules {
    pub fn new() -> LuaRules {
        let state = lua::State::new();
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

    pub fn load(&self, _: &Path) -> Result<(), Error> {
        return Ok(());
    }

    pub fn rules(&self) -> &[Rule] {
        return &[];
    }

    pub fn candidates(&self) -> &HashMap<String, Vec<Rule>> {
        return &self.candidates_rules;
    }
}
