use emu::emu_engine::EmuEffects;
use emu::vmstate::VmState;

pub trait RuleVerifier {
    fn verify(&self,
              args: &[u64],
              effects: EmuEffects,
              vmstate: &VmState)
              -> bool;
}

pub struct TargetRules {
    pub inputs: Vec<Vec<u64>>,
    pub verifier: Box<RuleVerifier>,
}

impl RuleVerifier for TargetRules {
    fn verify(&self,
              args: &[u64],
              effects: EmuEffects,
              vmstate: &VmState)
              -> bool {
        return self.verifier.verify(args, effects, vmstate);
    }
}
