use emu::Error;
use std::rc::Rc;
use unicorn;
use unicorn::unicorn_const::HookType;

pub struct Debugger {
    hook: unicorn::uc_hook,
    engine: Rc<unicorn::Unicorn>,
}

extern "C" fn on_code(_: unicorn::uc_handle,
                      address: u64,
                      size: u32,
                      _: *mut u64) {
    println!("0x{:016x}: [{}]", address, size);
}

impl Debugger {
    pub fn attach(engine: Rc<unicorn::Unicorn>) -> Result<Debugger, Error> {
        let hook = try!(engine.add_code_hook(HookType::CODE, 0, 0, on_code));
        return Ok(Debugger {
            hook: hook,
            engine: engine,
        });
    }

    pub fn detach(self) -> Result<(), Error> {
        try!(self.engine.remove_hook(self.hook));
        return Ok(());
    }
}
