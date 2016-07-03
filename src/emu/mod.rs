pub mod args;
pub mod calling_convention;
pub mod datatypes;
pub mod debugger;
pub mod emu_engine;
pub mod loader;
pub mod object_info;
pub mod vmstate;

use std::path::Path;

pub const EMUDATA_SIZE: usize = 0x10000;
pub const EMUDATA_ADDR: u64 = 0x11000000;
pub const STACK_SIZE: usize = 0x10000;
pub const STACK_ADDR: u64 = 0x10000000;

#[derive(Debug)]
pub enum Error {
    UnicornError(::unicorn::unicorn_const::Error),
    MapAlreadyExists,
    StackUninitialized,
    EmuDataUninitialized,
    ExecError(::unicorn::unicorn_const::Error),
    NotImplemented,
}

impl ::std::convert::From<::unicorn::unicorn_const::Error> for Error {
    fn from(e: ::unicorn::Error) -> Error {
        return Error::UnicornError(e);
    }
}

/// Helper function to create and initialize an emulation context from an elf
/// binary.
pub fn from_elf(path: &Path) -> Result<emu_engine::EmuEngine, loader::Error> {
    let vmstate = try!(self::loader::elf::load(path));
    let emu = try!(emu_engine::EmuEngine::new(vmstate));
    return Ok(emu);
}
