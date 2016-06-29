pub mod emu_engine;

use std::path::Path;

/// Helper function to create and initialize an emulation context from an elf
/// binary.
pub fn from_elf(path: &Path) -> emu_engine::EmuEngine {
  return emu_engine::EmuEngine {};
}
