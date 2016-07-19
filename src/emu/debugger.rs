use capstone;
use emu::Error;
use std::rc::Rc;
use std::cell::RefCell;
use unicorn;
use unicorn::unicorn_const::{CodeHookType, MemHookType};

pub struct Debugger {
    code_hook: Option<unicorn::uc_hook>,
    mem_hook: Option<unicorn::uc_hook>,
    engine: Rc<RefCell<unicorn::Unicorn>>,
}

pub fn attach(engine: Rc<RefCell<unicorn::Unicorn>>) -> Result<Debugger, Error> {
    let mut debugger = Debugger::new(engine);
    return debugger.attach().and(Ok(debugger));
}

impl Debugger {
    pub fn new(engine: Rc<RefCell<unicorn::Unicorn>>) -> Debugger {
        return Debugger {
            code_hook: None,
            mem_hook: None,
            engine: engine,
        };
    }

    pub fn attach(&mut self) -> Result<(), Error> {
        if let Some(_) = self.code_hook {
            return Ok(());
        }

        self.code_hook = Some(try!(self.engine
            .borrow_mut()
            .add_code_hook(CodeHookType::CODE,
                           1,
                           0,
                           |engine, address, size| {
                               Debugger::on_code(engine, address, size)
                           })));
        self.mem_hook = Some(try!(self.engine
            .borrow_mut()
            .add_mem_hook(MemHookType::MEM_INVALID,
                          1,
                          0,
                          |engine, mem_type, address, size, value| {
                Debugger::on_mem(engine, mem_type, address, size, value)
            })));
        return Ok(());
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        let mut engine = self.engine.borrow_mut();
        if let Some(code_hook) = self.code_hook {
            try!(engine.remove_hook(code_hook));
            self.code_hook = None;
        }
        if let Some(mem_hook) = self.mem_hook {
            try!(engine.remove_hook(mem_hook));
            self.mem_hook = None;
        }
        return Ok(());
    }

    fn on_code(engine: &unicorn::Unicorn, address: u64, size: u32) {
        let cs = capstone::Capstone::new(capstone::CsArch::ARCH_X86,
                                         capstone::CsMode::MODE_64)
            .expect("Failed to init capstone");
        let code = engine.mem_read(address, size as usize)
            .expect("Failed to read code memory");

        let inst_fmt = match cs.disasm(&code, address, 1) {
            Ok(insts) => {
                match insts.iter().next() {
                    Some(inst) => format!("{}", inst),
                    None => String::from("<none>"),
                }
            }
            Err(e) => format!("<err: {:?}", e),
        };

        println!("{}", inst_fmt);
    }
    fn on_mem(_: &unicorn::Unicorn,
              mem_type: unicorn::unicorn_const::MemType,
              address: u64,
              size: usize,
              value: i64)
              -> bool {
        println!("{:?} - 0x{:016x}: {} [{}]", mem_type, address, value, size);
        return false;
    }
}
