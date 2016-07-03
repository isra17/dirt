use emu::Error;
use std::rc::Rc;
use unicorn;
use unicorn::unicorn_const::HookType;

pub struct Debugger {
    code_hook: unicorn::uc_hook,
    mem_hook: unicorn::uc_hook,
    engine: Rc<unicorn::Unicorn>,
}

extern "C" fn on_code(_: unicorn::uc_handle,
                      address: u64,
                      size: u32,
                      _: *mut u64) {
    println!("0x{:016x}: [{}]", address, size);
}

extern "C" fn on_mem(_: unicorn::uc_handle,
                     mem_type: unicorn::unicorn_const::MemType,
                     address: u64,
                     size: i32,
                     value: i64,
                     _: *mut u64) {
    println!("{:?} - 0x{:016x}: {} [{}]", mem_type, address, value, size);
}

impl Debugger {
    pub fn attach(engine: Rc<unicorn::Unicorn>) -> Result<Debugger, Error> {
        let code_hook =
            try!(engine.add_code_hook(HookType::CODE, 1, 0, on_code));
        let mem_hook =
            try!(engine.add_mem_hook(HookType::MEM_READ_UNMAPPED, 1, 0, on_mem));

        return Ok(Debugger {
            code_hook: code_hook,
            mem_hook: mem_hook,
            engine: engine,
        });
    }

    pub fn detach(self) -> Result<(), Error> {
        try!(self.engine.remove_hook(self.code_hook));
        try!(self.engine.remove_hook(self.mem_hook));
        return Ok(());
    }
}
