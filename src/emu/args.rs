use emu::Error;
use emu::vmstate::{DataWriter, VmState};
use emu::datatypes::DataType;
use std::rc::Rc;

pub struct EmuArgs {
    argv: Vec<Rc<DataType>>,
}

pub struct PushableArg(Rc<DataType>, u64);

pub struct PushableArgs {
    argv: Vec<PushableArg>,
}

impl EmuArgs {
    pub fn new(argv: Vec<Rc<DataType>>) -> EmuArgs {
        return EmuArgs { argv: argv };
    }

    pub fn as_pushable(&self,
                       vmstate: &VmState)
                       -> Result<PushableArgs, Error> {
        let mut data_writer = DataWriter::new(vmstate);
        return Ok(PushableArgs {
            argv: try!(self.argv
                .iter()
                .map(|a| {
                    Ok(PushableArg(a.clone(),
                                   try!(a.pushable_value(&mut data_writer))))
                })
                .collect()),
        });
    }
}

impl PushableArgs {
    pub fn pushed_args(&self) -> Vec<u64> {
        return self.argv.iter().map(|&PushableArg(_, v)| v).collect();
    }
}
