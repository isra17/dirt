pub mod ruleset;
pub mod target_rules;

use emu::args::EmuArgs;
use emu::emu_engine::EmuEffects;
use emu::vmstate::VmState;
use self::target_rules::{RuleVerifier, TargetRules};
use std::rc::Rc;

struct AtoiRule;
impl RuleVerifier for AtoiRule {
    fn verify(&self, effects: &EmuEffects, vmstate: &VmState) -> bool {
        let pushed_args = effects.args.pushed_args();
        let a = match vmstate.read_str(pushed_args[0]) {
            Ok(x) => x,
            Err(_) => return false,
        };

        let n: i32 = a.parse().unwrap();

        return effects.return_value as i32 == n;
    }
}

pub fn fixtures() -> ruleset::RuleSet {
    use emu::datatypes::StringData;
    let inputs = vec![EmuArgs::new(vec![Rc::new(StringData::new("123"))]),
                      EmuArgs::new(vec![Rc::new(StringData::new("-12"))])];

    let atoi_rules = TargetRules {
        name: String::from("atoi"),
        inputs: inputs,
        verifier: Box::new(AtoiRule {}),
    };

    let ruleset = ruleset::RuleSet { candidates_rules: vec![atoi_rules] };
    return ruleset;
}
