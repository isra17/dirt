use byteorder::{ByteOrder, LittleEndian};
use emu;
use emu::Error;
use emu::debugger::Debugger;
use emu::env::{Env, Kernel};
use emu::object_info::MemMap;
use emu::vmstate::{DataWriter, VmState};
use std::cell::RefCell;
use rand::{Rng, StdRng};
use std::rc::Rc;
use unicorn::{Unicorn, uc_hook};
use unicorn::unicorn_const::{PROT_READ, PROT_WRITE};
use unicorn::{InsnSysX86, RegisterX86};

enum Msr {
    FS = 0xC0000100, // GS = 0xC0000101,
}

pub fn init_state(vmstate: &mut VmState) -> Result<(), Error> {
    // Set up the program stack as it would look from the kernel and
    // emulate <__start> up to <main>. This should give us a nice
    // initialized program state, if it worked...

    // TODO
    let kernel_map_addr = try!(vmstate.mem_map(MemMap {
        addr: emu::KERNEL_ADDR,
        size: emu::KERNEL_SIZE,
        flags: PROT_READ | PROT_WRITE,
        name: String::from("[kernel]"),
    }));
    let mut kernel_writer = DataWriter::new(vmstate, kernel_map_addr);

    try!(init_stack(vmstate, &mut kernel_writer));

    // Emulate up to main.
    let start_fva = vmstate.object_info
        .symbols
        .get("_start")
        .expect("_start not found")
        .value;
    let main_fva =
        vmstate.object_info.symbols.get("main").expect("main not found").value;

    // let mut debugger = Debugger::new(vmstate.engine.clone());
    // debugger.attach().expect("Failed to attach debugger");
    vmstate.engine
        .borrow()
        .emu_start(start_fva, main_fva, emu::EMU_TIMEOUT, emu::EMU_MAXCOUNT)
        .expect("Failed to run up to main");
    // debugger.detach().expect("Failed to detach debugger");

    return Ok(());
}

#[allow(non_camel_case_types)]
pub enum AuxVecType {
    ELF_AT_NULL = 0,
    ELF_AT_IGNORE,
    ELF_AT_EXECFD,
    ELF_AT_PHDR,
    ELF_AT_PHENT,
    ELF_AT_PHNUM,
    ELF_AT_PAGESZ,
    ELF_AT_BASE,
    ELF_AT_FLAGS,
    ELF_AT_ENTRY,
    ELF_AT_NOTELF,
    ELF_AT_UID,
    ELF_AT_EUID,
    ELF_AT_GID,
    ELF_AT_EGID,
    ELF_AT_PLATFORM,
    ELF_AT_HWCAP,
    ELF_AT_CLKTCK,
    ELF_AT_RANDOM = 25,
    ELF_AT_SYSINFO = 32,
    ELF_AT_SYSINFO_EHDR,
}

struct AuxVec(AuxVecType, u64);

fn get_auxv(vmstate: &VmState) -> Vec<AuxVec> {
    let mut rng = StdRng::new().expect("Failed to initialize RNG");
    let mut rand = [0u8; 16];
    rng.fill_bytes(&mut rand);
    let rand_addr = vmstate.sp().unwrap() - rand.len() as u64;
    vmstate.set_sp(rand_addr).expect("Failed to push rand");
    vmstate.engine
        .borrow()
        .mem_write(rand_addr, &rand)
        .expect("failed to write rand");

    return vec![
        AuxVec(AuxVecType::ELF_AT_NULL, 0),
        AuxVec(AuxVecType::ELF_AT_RANDOM, rand_addr),
        AuxVec(AuxVecType::ELF_AT_EGID, 0),
        AuxVec(AuxVecType::ELF_AT_GID, 0),
        AuxVec(AuxVecType::ELF_AT_EUID, 0),
        AuxVec(AuxVecType::ELF_AT_UID, 0),
        AuxVec(AuxVecType::ELF_AT_FLAGS, 0),
        AuxVec(AuxVecType::ELF_AT_PAGESZ, 0x1000),
    ];
}

fn init_stack(vmstate: &VmState,
              data_writer: &mut DataWriter)
              -> Result<(), Error> {
    // auxv
    for AuxVec(auxv_type, value) in get_auxv(vmstate) {
        try!(vmstate.stack_push(value));
        try!(vmstate.stack_push(auxv_type as u64));
    }
    // env
    try!(vmstate.stack_push(0));
    // argv
    try!(vmstate.stack_push(0));
    try!(vmstate.stack_push(try!(data_writer.write_str("/emu"))));

    // argc
    try!(vmstate.stack_push(1));

    return Ok(());
}

pub struct LinuxKernel {
    intr_hook: Option<uc_hook>,
    brk_ptr: u64,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Syscall {
    Open = 2,
    Mmap = 9,
    Brk = 12,
    Writev = 20,
    Uname = 63,
    Prctl = 158,
}

#[allow(dead_code)]
#[derive(Debug)]
enum PrctlCode {
    ArchSetGs = 0x1001,
    ArchSetFs = 0x1002,
    ArchGetFs = 0x1003,
    ArchGetGs = 0x1004,
}

// TODO: Implements the following as a trait for Unicorn. Duplicates current
// VmState logic...
fn read_str(engine: &Unicorn, addr: u64) -> Result<String, Error> {
    // TODO: Read a page at once, should be faster.
    let mut data_buf: Vec<u8> = vec![];
    let mut i = addr;
    loop {
        match try!(engine.mem_read(i, 1)).pop() {
            None => break,
            Some(0) => break,
            Some(b) => data_buf.push(b),
        }

        i += 1;
    }
    return String::from_utf8(data_buf).map_err(|e| Error::FromUtf8Error(e));
}

fn read_usize(engine: &Unicorn, addr: u64) -> Result<u64, Error> {
    // TODO: Make it arch independant.
    return Ok(LittleEndian::read_u64(&try!(engine.mem_read(addr, 8))));
}

fn set_fs(engine: &Unicorn, fs: u64, rip: u64) -> Result<(), Error> {
    let saved_rax = engine.reg_read(RegisterX86::RAX as i32).unwrap();
    let saved_rcx = engine.reg_read(RegisterX86::RCX as i32).unwrap();
    let saved_rdx = engine.reg_read(RegisterX86::RDX as i32).unwrap();

    // Push saved rip.
    let sp = engine.reg_read(RegisterX86::RSP as i32).unwrap();
    try!(engine.reg_write(RegisterX86::RSP as i32, sp - 8));
    let mut buf = Vec::with_capacity(8);
    buf.resize(8, 0);
    LittleEndian::write_u64(&mut buf, rip);
    try!(engine.mem_write(sp - 8, &buf));


    // println!("Setting FS as {:x}", fs);
    try!(engine.reg_write(RegisterX86::RAX as i32, fs & 0xffffffff));
    try!(engine.reg_write(RegisterX86::RDX as i32, (fs >> 32) & 0xffffffff));
    try!(engine.reg_write(RegisterX86::RCX as i32, Msr::FS as u64));
    let mut shellcode = vec![0x0f, 0x30];

    // mov rax, saved_rax
    shellcode.append(&mut vec![0x48, 0xB8]);
    buf.resize(8, 0);
    LittleEndian::write_u64(&mut buf, saved_rax);
    shellcode.append(&mut buf);
    // mov rcx, saved_rcx
    shellcode.append(&mut vec![0x48, 0xB9]);
    buf.resize(8, 0);
    LittleEndian::write_u64(&mut buf, saved_rcx);
    shellcode.append(&mut buf);
    // mov rdx, saved_rdx
    shellcode.append(&mut vec![0x48, 0xBA]);
    buf.resize(8, 0);
    LittleEndian::write_u64(&mut buf, saved_rdx);
    shellcode.append(&mut buf);
    // ret
    shellcode.push(0xC3);

    return run_shellcode(engine, &shellcode);
}

fn run_shellcode(engine: &Unicorn, code: &[u8]) -> Result<(), Error> {
    // Dirty hack to hijack control flow...
    let addr = emu::SHELLCODE_ADDR;

    try!(engine.mem_write(addr, code));
    try!(engine.reg_write(RegisterX86::RIP as i32, addr - 2));

    // println!("shellcode: {:?}", engine.mem_read(addr, code.len()));
    // println!("rip: {:x}",
    //         engine.reg_read(RegisterX86::RIP as i32).unwrap());

    return Ok(());
}

impl LinuxKernel {
    pub fn on_syscall(&mut self, engine: &Unicorn) {
        let rip = engine.reg_read(RegisterX86::RIP as i32).unwrap();
        let sysno = engine.reg_read(RegisterX86::RAX as i32).unwrap();
        // println!("syscall({}) at {:x}", sysno, rip);

        let argv = vec![
            engine.reg_read(RegisterX86::RDI as i32).unwrap(),
            engine.reg_read(RegisterX86::RSI as i32).unwrap(),
            engine.reg_read(RegisterX86::RDX as i32).unwrap(),
            engine.reg_read(RegisterX86::R10 as i32).unwrap(),
            engine.reg_read(RegisterX86::R8 as i32).unwrap(),
            engine.reg_read(RegisterX86::R9 as i32).unwrap(),
        ];

        let result = match sysno {
            n if n == Syscall::Open as u64 => {
                // println!("Open(\"{}\")", read_str(engine, argv[0]).unwrap());
                0xffffffffffffffff
            }
            n if n == Syscall::Brk as u64 => {
                // println!("Brk(0x{:x})", argv[0]);
                let ptr = argv[0];
                if ptr == 0 {
                    self.brk_ptr
                } else {
                    self.brk_ptr = ptr;
                    ptr
                }
            }
            n if n == Syscall::Writev as u64 => {
                // println!("writev({}, 0x{:x}, {})", argv[0], argv[1],
                // argv[2]);
                for n in 0..argv[2] {
                    let addr = read_usize(engine, argv[1] + 0x10 * n)
                        .expect("addr");
                    let size = read_usize(engine, argv[1] + 8 + 0x10 * n)
                        .expect("size");
                    let data = engine.mem_read(addr, size as usize)
                        .expect("data");
                    // println!("{:?} > 2", String::from_utf8_lossy(&data));
                }
                0xffffffffffffffff
            }
            n if n == Syscall::Uname as u64 => {
                fn extend_64_bytes(data: &[u8]) -> Vec<u8> {
                    let mut bytes = Vec::from(data);
                    bytes.resize(64, 0);
                    bytes
                }

                // println!("uname(0x{:x})", argv[0]);
                let mut uname = Vec::with_capacity(64 * 6);
                uname.append(&mut extend_64_bytes("Linux".as_bytes()));
                uname.append(&mut extend_64_bytes("dirt".as_bytes()));
                uname.append(&mut extend_64_bytes("4.6.2-1-ARCH".as_bytes()));
                uname.append(&mut extend_64_bytes("#1 SMP PREEMPT Wed Jun 8 \
                                                   08:40:59 CEST 2016"
                    .as_bytes()));
                uname.append(&mut extend_64_bytes("x86_64".as_bytes()));
                uname.append(&mut extend_64_bytes("GNU/Linux".as_bytes()));
                engine.mem_write(argv[0], &uname)
                    .expect("Failed to write uname data");
                0
            }
            n if n == Syscall::Prctl as u64 => {
                let code = argv[0];
                let addr = argv[1];
                // println!("prctl(0x{:x}, 0x{:x})", argv[0], argv[1]);
                match code {
                    n if n == PrctlCode::ArchSetFs as u64 => {
                        let next_rip = rip + 2;
                        engine.reg_write(RegisterX86::RAX as i32, 0)
                            .expect("Failed to set rax");
                        set_fs(engine, addr, next_rip)
                            .expect("Failed to set fs");
                        return;
                    }
                    _ => panic!("Prctl code not implemented"),
                };
            }
            _ => 0,
        };

        engine.reg_write(RegisterX86::RAX as i32, result)
            .expect("Failed to set rax");
    }
}

pub struct LinuxEnv {
}

impl Env for LinuxEnv {
    fn attach(&self, vmstate: &mut VmState) -> Rc<RefCell<Kernel>> {
        let kernel = Rc::new(RefCell::new(LinuxKernel {
            intr_hook: None,
            brk_ptr: emu::BRK_ADDR,
        }));

        vmstate.mem_map(MemMap {
                addr: kernel.borrow().brk_ptr,
                size: emu::BRK_SIZE,
                flags: PROT_READ | PROT_WRITE,
                name: String::from("[heap]"),
            })
            .expect("Failed to map heap");

        let hook_kernel = kernel.clone();
        kernel.borrow_mut().intr_hook = Some(vmstate.engine
            .borrow_mut()
            .add_insn_sys_hook(InsnSysX86::SYSCALL, 1, 0, move |engine| {
                hook_kernel.borrow_mut()
                    .on_syscall(engine);
            })
            .expect("Fail to hook interrupts"));
        return kernel;
    }
}

impl Kernel for LinuxKernel {
    fn reset(&mut self) -> Result<(), Error> {
        self.brk_ptr = emu::BRK_ADDR;
        Ok(())
    }

    fn detach(&mut self, vmstate: &mut VmState) {
        if let Some(intr_hook) = self.intr_hook {
            vmstate.engine
                .borrow_mut()
                .remove_hook(intr_hook)
                .expect("Failed to remove hook");
            self.intr_hook = None;
        }
    }
}
