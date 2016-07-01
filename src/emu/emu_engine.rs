use unicorn;
use unicorn::CpuX86;

use dirt_engine::TargetInfo;

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
    pub uc: CpuX86,
}

impl EmuEngine {
    pub fn new() -> EmuEngine {
        let uc = CpuX86::new(unicorn::Mode::MODE_32)
            .expect("failed to instantiate emulator");
        return EmuEngine { uc: uc };
    }

    pub fn call(&self,
                target: &TargetInfo,
                args: &EmuArgs)
                -> Result<EmuEffects, EmuError> {
        // return Ok(EmuEffects { return_value: 0 });
        return Err(EmuError::UnicornError);
    }
}
