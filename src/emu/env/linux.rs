use emu;
use emu::Error;
use emu::env::{Env, Kernel};
use emu::object_info::MemMap;
use emu::vmstate::VmState;
use std::cell::RefCell;
use std::rc::Rc;
use unicorn::{Unicorn, uc_hook};
use unicorn::unicorn_const::{PROT_READ, PROT_WRITE};
use unicorn::{InsnSysX86, RegisterX86};

enum Msr {
    FS = 0xC0000100, // GS = 0xC0000101,
}

pub fn init_state(vmstate: &mut VmState) -> Result<(), Error> {
    // Set up TLS.
    try!(init_tls(vmstate));

    // Set up the program stack as it would look from the kernel and
    // emulate <__start> up to <main>. This should give us a nice
    // initialized program state, if it worked...

    // TODO
    // let kernel_map_addr = try!(vmstate.mem_map(MemMap {
    // addr: emu::KERNEL_ADDR,
    // size: emu::KERNEL_SIZE,
    // flags: PROT_READ | PROT_WRITE,
    // name: String::from("[kernel]"),
    // }));
    // let mut kernel_writer = DataWriter::new(vmstate, kernel_map_addr);
    //
    // try!(init_stack(vmstate, &mut kernel_writer));
    //
    // Emulate up to main.
    // let start_fva = vmstate.object_info
    // .symbols
    // .get("_start")
    // .expect("_start not found")
    // .value;
    // let main_fva =
    // vmstate.object_info.symbols.get("main").expect("main not found").value;
    // try!(vmstate.engine
    // .emu_start(start_fva, main_fva, emu::EMU_TIMEOUT, emu::EMU_MAXCOUNT));
    //
    return Ok(());
}

fn init_tls(vmstate: &mut VmState) -> Result<(), Error> {
    // Map TLS.
    try!(vmstate.mem_map(MemMap {
        addr: emu::TLS_ADDR,
        size: emu::TLS_SIZE,
        flags: PROT_READ | PROT_WRITE,
        name: String::from("[tls]"),
    }));

    let fs = emu::TLS_ADDR + 0x1000;
    let engine = vmstate.engine.borrow();

    // Set FS.
    try!(engine.reg_write(RegisterX86::RAX as i32, fs & 0xffffffff));
    try!(engine.reg_write(RegisterX86::RDX as i32, (fs >> 32) & 0xffffffff));
    try!(engine.reg_write(RegisterX86::RCX as i32, Msr::FS as u64));
    try!(vmstate.run_shellcode(&[0x0F, 0x30]));
    return Ok(());
}

// fn init_stack(vmstate: &VmState,
// data_writer: &mut DataWriter)
// -> Result<(), Error> {
// auxv
// try!(vmstate.stack_push(0));
// for AuxVec(auxv_type, value) in get_auxv() {
// try!(vmstate.stack_push(auxv_type as u64));
// try!(vmstate.stack_push(value));
// }
// env
// try!(vmstate.stack_push(0));
// argv
// try!(vmstate.stack_push(0));
// try!(vmstate.stack_push(try!(data_writer.write_str("/emu"))));
//
// argc
// try!(vmstate.stack_push(1));
//
// return Ok(());
// }
//
// #[allow(non_camel_case_types)]
// pub enum AuxVecType {
// ELF_AT_NULL = 0,
// ELF_AT_IGNORE,
// ELF_AT_EXECFD,
// ELF_AT_PHDR,
// ELF_AT_PHENT,
// ELF_AT_PHNUM,
// ELF_AT_PAGESZ,
// ELF_AT_BASE,
// ELF_AT_FLAGS,
// ELF_AT_ENTRY,
// ELF_AT_NOTELF,
// ELF_AT_UID,
// ELF_AT_EUID,
// ELF_AT_GID,
// ELF_AT_EGID,
// ELF_AT_PLATFORM,
// ELF_AT_HWCAP,
// ELF_AT_CLKTCK,
// ELF_AT_RANDOM = 25,
// ELF_AT_SYSINFO = 32,
// ELF_AT_SYSINFO_EHDR,
// }
//
// struct AuxVec(AuxVecType, u64);
//
// fn get_auxv() -> Vec<AuxVec> {
// return vec![
// AuxVec(AuxVecType::ELF_AT_PAGESZ, 0x1000),
// AuxVec(AuxVecType::ELF_AT_FLAGS, 0),
// ];
// }
//

pub struct LinuxKernel {
    intr_hook: Option<uc_hook>,
    brk_ptr: u64,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Syscall {
    Mmap = 0x9,
    Brk = 0xc,
}

impl LinuxKernel {
    pub fn on_syscall(&mut self, engine: &Unicorn) {
        let sysno = engine.reg_read(RegisterX86::RAX as i32).unwrap();
        let argv = vec![
            engine.reg_read(RegisterX86::RDI as i32).unwrap(),
            engine.reg_read(RegisterX86::RSI as i32).unwrap(),
            engine.reg_read(RegisterX86::RDX as i32).unwrap(),
            engine.reg_read(RegisterX86::R10 as i32).unwrap(),
            engine.reg_read(RegisterX86::R8 as i32).unwrap(),
            engine.reg_read(RegisterX86::R9 as i32).unwrap(),
        ];

        let result = match sysno {
            n if n == Syscall::Brk as u64 => {
                println!("syscall({}): Brk(0x{:x})", sysno, argv[0]);
                let ptr = argv[0];
                if ptr == 0 {
                    self.brk_ptr
                } else {
                    self.brk_ptr = ptr;
                    ptr
                }
            }
            _ => {
                println!("syscall({})", sysno);
                0
            }
        };

        engine.reg_write(RegisterX86::RAX as i32, result)
            .expect("Failed to set rax");
    }
}

pub struct LinuxEnv {
}

impl Env for LinuxEnv {
    fn attach(&self, vmstate: &mut VmState) -> Rc<RefCell<Kernel>> {
        println!("Hooking syscall");
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
