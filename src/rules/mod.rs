pub mod ruleset;
pub mod target_rules;

use emu::emu_engine::EmuEffects;
use emu::vmstate::VmState;
use self::target_rules::{RuleVerifier, TargetRules};

struct StrcmpRule;
impl RuleVerifier for StrcmpRule {
    fn verify(&self,
              args: &[u64],
              effects: EmuEffects,
              vmstate: &VmState)
              -> bool {
        let a = match vmstate.read_str(args[0]) {
            Ok(x) => x,
            Err(_) => return false,
        };

        let b = match vmstate.read_str(args[1]) {
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
    let inputs = vec![vec![0, 0], vec![0x90000000, 0]];

    let strcmp_rules = TargetRules {
        inputs: inputs,
        verifier: Box::new(StrcmpRule {}),
    };

    let ruleset = ruleset::RuleSet { candidates_rules: vec![strcmp_rules] };
    return ruleset;
}
