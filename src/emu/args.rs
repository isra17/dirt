use emu::Error;
use emu::vmstate::VmState;
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
        let mut data_writer = try!(vmstate.emudata_writer());
        let argv: Result<Vec<_>, Error> = self.argv
            .iter()
            .map(|a| {
                Ok(PushableArg(a.clone(),
                               try!(a.pushable_value(&mut data_writer))))
            })
            .collect();
        return Ok(PushableArgs { argv: try!(argv) });
    }
}

impl PushableArgs {
    pub fn pushed_args(&self) -> Vec<u64> {
        return self.argv.iter().map(|&PushableArg(_, v)| v).collect();
    }

    pub fn nth(&self, n: usize) -> u64 {
        return self.argv[n].1;
    }
}
