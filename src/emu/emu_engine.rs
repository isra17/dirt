use std::rc::Rc;
use unicorn;
use unicorn::CpuX86;

use dirt_engine::TargetInfo;
use emu::vmstate::VmState;

pub enum EmuError {
    UnicornError,
}

pub struct EmuEffects {
    return_value: u64,
}

pub struct EmuArgs {
    argv: Vec<u64>,
}

pub struct EmuEngine {
    pub uc: Rc<unicorn::CpuX86>,
    vmstate: VmState,
}

impl EmuEngine {
    pub fn new() -> EmuEngine {
        let uc = Rc::new(CpuX86::new(unicorn::Mode::MODE_32)
            .expect("failed to create emulator"));
        return EmuEngine {
            uc: uc.clone(),
            vmstate: VmState::new(uc.clone()),
        };
    }

    pub fn call(&self,
                target: &TargetInfo,
                args: &EmuArgs)
                -> Result<EmuEffects, EmuError> {
        // return Ok(EmuEffects { return_value: 0 });
        return Err(EmuError::UnicornError);
    }
}
