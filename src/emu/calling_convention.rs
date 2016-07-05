use emu::Error;
use emu::vmstate::VmState;
use dirt_engine::CallingConvention as CCEnum;
use unicorn::x86_const::RegisterX86;

pub trait CallingConvention {
    fn init_args(&self, args: &[u64], vmstate: &VmState) -> Result<(), Error>;
}

struct Stdcall;
struct SystemV;

impl CallingConvention for Stdcall {
    fn init_args(&self, args: &[u64], vmstate: &VmState) -> Result<(), Error> {
        for arg in args {
            try!(vmstate.stack_push(*arg));
        }
        return Ok(());
    }
}

impl CallingConvention for SystemV {
    fn init_args(&self, args: &[u64], vmstate: &VmState) -> Result<(), Error> {
        let mut args_iter = args.into_iter();
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::RDI as i32, *arg));
        }
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::RSI as i32, *arg));
        }
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::RDX as i32, *arg));
        }
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::RCX as i32, *arg));
        }
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::R8 as i32, *arg));
        }
        if let Some(arg) = args_iter.next() {
            try!(vmstate.engine.reg_write(RegisterX86::R9 as i32, *arg));
        }

        for arg in args_iter {
            try!(vmstate.stack_push(*arg));
        }

        return Ok(());
    }
}

pub fn new(cc: &CCEnum) -> Box<CallingConvention> {
    return match cc {
        &CCEnum::Stdcall => Box::new(Stdcall {}),
        &CCEnum::SystemV => Box::new(SystemV {}),
    };
}
