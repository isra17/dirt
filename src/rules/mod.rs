pub mod ruleset;
pub mod target_rules;

use emu::args::EmuArgs;
use emu::emu_engine::EmuEffects;
use emu::vmstate::VmState;
use self::target_rules::{RuleVerifier, TargetRules};
use std::rc::Rc;

struct StrcmpRule;
impl RuleVerifier for StrcmpRule {
    fn verify(&self, effects: &EmuEffects, vmstate: &VmState) -> bool {
        let pushed_args = effects.args.pushed_args();
        let a = match vmstate.read_str(pushed_args[0]) {
            Ok(x) => x,
            Err(_) => return false,
        };

        let b = match vmstate.read_str(pushed_args[1]) {
            Ok(x) => x,
            Err(_) => return false,
        };

        if a == b {
            return effects.return_value == 0;
        } else {
            return effects.return_value != 0;
        }
    }
}

pub fn fixtures() -> ruleset::RuleSet {
    use emu::datatypes::StringData;
    let inputs = vec![EmuArgs::new(vec![Rc::new(StringData::new("a")),
                                        Rc::new(StringData::new("a"))]),
                      EmuArgs::new(vec![Rc::new(StringData::new("a")),
                                        Rc::new(StringData::new("b"))])];

    let strcmp_rules = TargetRules {
        name: String::from("strcmp"),
        inputs: inputs,
        verifier: Box::new(StrcmpRule {}),
    };

    let ruleset = ruleset::RuleSet { candidates_rules: vec![strcmp_rules] };
    return ruleset;
}
