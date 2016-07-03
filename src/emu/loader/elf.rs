use utils::LogError;
use emu::loader::Error;
use emu::object_info::MemMap;
use emu::vmstate::VmState;
use elf;
use std::io;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use unicorn;

/// Align a memory size.
fn aligned_size(size: usize, page_size: usize) -> usize {
    return (size / page_size + 1) * page_size;
}

/// Align a memory address.
fn aligned_addr(addr: u64, page_size: u64) -> u64 {
    return (addr / page_size) * page_size;
}

/// Convert ::elf::types::ProgFlag to ::unicorn::Permission.
fn prot_from_elf_flags(flag: elf::types::ProgFlag)
                       -> unicorn::unicorn_const::Protection {
    return unicorn::unicorn_const::Protection::from_bits(flag.0)
        .expect("Cannot convert ELF flags to unicorn Protection");
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

    let emu = Rc::new(match try!(Arch::new(elf_file.ehdr.machine)) {
        Arch(arch, mode) => try!(unicorn::Unicorn::new(arch, mode)),
    });

    let mut vmstate = VmState::new(emu.clone());

    // unwrap, we open it once, should open again...
    let mut file_stream = File::open(path).unwrap();

    // Load segment in emulator.
    let loadable_segments =
        elf_file.phdrs.iter().filter(|s| s.progtype == elf::types::PT_LOAD);
    for phdr in loadable_segments {
        let page_addr = aligned_addr(phdr.vaddr, 0x1000);
        let offset = (phdr.vaddr - page_addr) as usize;
        let page_size = aligned_size(phdr.memsz as usize + offset, 0x1000);
        let flags = prot_from_elf_flags(phdr.flags);
        try!(vmstate.mem_map(MemMap {
                addr: page_addr,
                size: page_size,
                flags: flags,
                name: String::new(),
            })
            .log_err(|_| format!("Failed to map segment: {:?}", phdr)));

        try!(file_stream.seek(io::SeekFrom::Start(phdr.offset))
            .log_err(|_| {
                format!("Failed to seek to segment offset: {:?}", phdr)
            }));

        let mut data_buf = Vec::with_capacity(phdr.filesz as usize);
        data_buf.resize(phdr.filesz as usize, 0);
        try!(file_stream.read_exact(data_buf.as_mut_slice())
            .log_err(|_| {
                format!("Failed to read segment content: {:?}", phdr)
            }));

        try!(emu.mem_write(phdr.vaddr, data_buf.as_slice())
            .log_err(|_| {
                format!("Failed to write segment to emulator: {:?}", phdr)
            }));
    }


    return Ok(vmstate);
}
