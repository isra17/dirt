extern crate unicorn;

use emu::loader::Error;
use emu::vmstate::VmState;
use elf;
use std::fs::File;
use std::io;
use std::path::Path;

/// Convert ::elf::types::ProgFlag to ::unicorn::Permission.
fn prot_from_elf_flags(flag: elf::types::ProgFlag)
                       -> unicorn::unicorn_const::Protection {
    return unicorn::unicorn_const::Protection::from_bits(flag.0).unwrap();
}

struct Arch(unicorn::unicorn_const::Arch, unicorn::unicorn_const::Mode);

impl Arch {
    /// Create a unicorn::Unicorn instance given a Machine's arch type.
    pub fn new(arch: elf::types::Machine) -> Result<Arch, Error> {
        use unicorn::unicorn_const::{Arch, Mode};
        match arch {
            elf::types::EM_386 => Ok(Arch(Arch::X86, Mode::MODE_32)),
            elf::types::EM_X86_64 => Ok(Arch(Arch::X86, Mode::MODE_64)),
            _ => Err(Error::UnsupportedArch(arch)),
        }
    }
}

pub fn load(path: &Path) -> Result<VmState, Error> {
    use std::io::{Read, Seek};

    let elf_file = try!(elf::File::open_path(path));

    let emu = match try!(Arch::new(elf_file.ehdr.machine)) {
        Arch(arch, mode) => try!(unicorn::Unicorn::new(arch, mode)),
    };

    // unwrap, we open it once, should open again...
    let mut file_stream = File::open(path).unwrap();

    // Load segment in emulator.
    let loadable_segments =
        elf_file.phdrs.iter().filter(|s| s.progtype == elf::types::PT_LOAD);
    for phdr in loadable_segments {
        try!(emu.mem_map(phdr.vaddr,
                         phdr.memsz as usize,
                         prot_from_elf_flags(phdr.flags)));

        try!(file_stream.seek(io::SeekFrom::Start(phdr.offset)));
        let mut data_buf = Vec::with_capacity(phdr.filesz as usize);
        try!(file_stream.read_exact(data_buf.as_mut_slice()));

        try!(emu.mem_write(phdr.vaddr, data_buf.as_slice()));
    }

    let vmstate = VmState::new(::std::rc::Rc::new(emu));

    return Ok(vmstate);
}
