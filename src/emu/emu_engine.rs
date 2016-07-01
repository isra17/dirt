
use unicorn;
use unicorn::CpuX86;

pub struct EmuEngine {
  pub uc: CpuX86,
}

impl EmuEngine {
  pub fn new() -> EmuEngine {
    let uc = CpuX86::new(unicorn::Mode::MODE_32)
      .expect("failed to instantiate emulator");
    return EmuEngine { uc: uc };
  }
}
