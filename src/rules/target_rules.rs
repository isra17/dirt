use emu::datatypes::DataType;
use emu::emu_engine::EmuEffects;
use emu::vmstate::VmState;
use emu::args::{EmuArgs, PushableArgs};

pub trait RuleVerifier {
    fn verify(&self, effects: &EmuEffects, vmstate: &VmState) -> bool;
}

pub struct TargetRules {
    pub inputs: Vec<EmuArgs>,
    pub verifier: Box<RuleVerifier>,
}

impl RuleVerifier for TargetRules {
    fn verify(&self, effects: &EmuEffects, vmstate: &VmState) -> bool {
        return self.verifier.verify(effects, vmstate);
    }
}
