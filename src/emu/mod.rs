pub mod args;
pub mod calling_convention;
pub mod datatypes;
pub mod debugger;
pub mod emu_engine;
pub mod env;
pub mod loader;
pub mod object_info;
pub mod vmstate;

use std::path::Path;

pub const STACK_ADDR: u64 = 0x10000000;
pub const STACK_SIZE: usize = 0x10000;
pub const EMUDATA_ADDR: u64 = 0x11000000;
pub const EMUDATA_SIZE: usize = 0x10000;
pub const KERNEL_ADDR: u64 = 0x12000000;
pub const KERNEL_SIZE: usize = 0x10000;
pub const TLS_ADDR: u64 = 0x13000000;
pub const TLS_SIZE: usize = 0x10000;
pub const SHELLCODE_ADDR: u64 = 0x14000000;
pub const SHELLCODE_SIZE: usize = 0x10000;
pub const BRK_ADDR: u64 = 0x20000000;
pub const BRK_SIZE: usize = 0x00010000;

pub const CODE_SENTINEL: u64 = 0x80000000;
pub const EMU_TIMEOUT: u64 = 1 * 1000 * 1000; // 1 sec.
pub const EMU_MAXCOUNT: usize = 0x10000;


#[derive(Debug)]
pub enum Error {
    UnicornError(::unicorn::unicorn_const::Error),
    MapAlreadyExists,
    StackUninitialized,
    EmuDataUninitialized,
    ExecError(::unicorn::unicorn_const::Error),
    FromUtf8Error(::std::string::FromUtf8Error),
    NotImplemented,
}

impl ::std::convert::From<::unicorn::unicorn_const::Error> for Error {
    fn from(e: ::unicorn::Error) -> Error {
        return Error::UnicornError(e);
    }
}

impl ::std::convert::From<::std::string::FromUtf8Error> for Error {
    fn from(e: ::std::string::FromUtf8Error) -> Error {
        return Error::FromUtf8Error(e);
    }
}

/// Helper function to create and initialize an emulation context from an elf
/// binary.
pub fn from_elf(path: &Path) -> Result<emu_engine::EmuEngine, loader::Error> {
    let vmstate = try!(self::loader::elf::load(path));
    let emu = try!(emu_engine::EmuEngine::new(vmstate));
    return Ok(emu);
}
