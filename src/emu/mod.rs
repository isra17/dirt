pub mod args;
pub mod calling_convention;
pub mod datatypes;
pub mod emu_engine;
pub mod loader;
pub mod vmstate;

use std::path::Path;

#[derive(Debug)]
pub enum Error {
    UnicornError(::unicorn::unicorn_const::Error),
    NotImplemented,
}

/// Helper function to create and initialize an emulation context from an elf
/// binary.
pub fn from_elf(path: &Path) -> Result<emu_engine::EmuEngine, loader::Error> {
    let vmstate = try!(self::loader::elf::load(path));
    let emu = emu_engine::EmuEngine::new(vmstate);
    return Ok(emu);
}
