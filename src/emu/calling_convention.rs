use emu::Error;
use emu::vmstate::VmState;
use dirt_engine::CallingConvention as CCEnum;

pub trait CallingConvention {
    fn init_args(&self, args: &[u64], vmstate: &VmState) -> Result<(), Error>;
}

struct Stdcall;

impl CallingConvention for Stdcall {
    fn init_args(&self, args: &[u64], vmstate: &VmState) -> Result<(), Error> {
        for arg in args {
            try!(vmstate.stack_push(*arg));
        }
        return Ok(());
    }
}

pub fn new(cc: &CCEnum) -> Box<CallingConvention> {
    return Box::new(match cc {
        &CCEnum::Stdcall => Stdcall {},
    });
}
