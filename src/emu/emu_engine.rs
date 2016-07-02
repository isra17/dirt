use dirt_engine::TargetInfo;
use emu::Error;
use emu::vmstate::VmState;

pub struct EmuEffects {
    return_value: u64,
}

pub struct EmuArgs {
    argv: Vec<u64>,
}

pub struct EmuEngine {
    pub vmstate: VmState,
}

const CODE_SENTINEL: u64 = 0x80000000;

impl EmuEngine {
    pub fn new(vmstate: VmState) -> EmuEngine {
        // Code sentinel used to trap function return.
        vmstate.engine
            .mem_map(CODE_SENTINEL, 1, ::unicorn::unicorn_const::PROT_EXEC)
            .unwrap();
        return EmuEngine { vmstate: vmstate };
    }

    pub fn call(&self,
                target: &TargetInfo,
                args: &EmuArgs)
                -> Result<EmuEffects, Error> {
        self.clean_state().expect("Cannot clean emulator state");

        let cc = ::emu::calling_convention::new(&target.cc);
        try!(cc.init_args(args.argv.as_slice(), &self.vmstate));
        try!(self.call_and_return(target.fva));

        return self.collect_call_results();
    }

    fn clean_state(&self) -> Result<(), Error> {
        return Err(Error::NotImplemented);
    }

    fn call_and_return(&self, ip: u64) -> Result<(), Error> {
        return self.vmstate.set_call(ip, CODE_SENTINEL);
    }

    fn collect_call_results(&self) -> Result<EmuEffects, Error> {
        return Err(Error::NotImplemented);
    }
}
