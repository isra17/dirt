pub mod elf;

use std::io;

#[derive(Debug)]
pub enum Error {
    ParseError(::elf::ParseError),
    EmuError(::unicorn::unicorn_const::Error),
    IoError(io::Error),
    UnsupportedArch(::elf::types::Machine),
}

// Bunch of helper to make it simpler to `try!` things not returning our error
// type.
impl ::std::convert::From<::unicorn::Error> for Error {
    fn from(e: ::unicorn::Error) -> Error {
        return Error::EmuError(e);
    }
}

impl ::std::convert::From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Error {
        return Error::IoError(e);
    }
}

impl ::std::convert::From<::elf::ParseError> for Error {
    fn from(e: ::elf::ParseError) -> Error {
        return Error::ParseError(e);
    }
}
