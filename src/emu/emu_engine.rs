use dirt_engine::TargetInfo;
use emu;
use emu::Error;
use emu::args::{EmuArgs, PushableArgs};
use emu::vmstate::VmState;

pub struct EmuEffects<'a> {
    pub vmstate: &'a VmState,
    pub return_value: u64,
    pub args: PushableArgs,
}

pub struct EmuEngine {
    pub vmstate: VmState,
}

impl EmuEngine {
    pub fn new(mut vmstate: VmState) -> Result<EmuEngine, Error> {
        // Code sentinel used to trap function return.
        vmstate.engine
            .borrow()
            .mem_map(emu::CODE_SENTINEL,
                     0x1000,
                     ::unicorn::unicorn_const::PROT_EXEC)
            .expect("Failed to map code sentinel");
        try!(vmstate.init());

        return Ok(EmuEngine { vmstate: vmstate });
    }

    pub fn call(&self,
                target: &TargetInfo,
                args: &EmuArgs)
                -> Result<EmuEffects, Error> {
        self.clean_state().expect("Cannot clean emulator state");

        let pushable_args = try!(args.as_pushable(&self.vmstate));
        let cc = ::emu::calling_convention::new(&target.cc);
        try!(cc.init_args(&pushable_args.pushed_args(), &self.vmstate));
        try!(self.call_and_return(target.fva));

        return self.vmstate.collect_call_results(pushable_args);
    }

    fn clean_state(&self) -> Result<(), Error> {
        try!(self.vmstate.reset_stack());
        try!(self.vmstate.reset_emudata());
        return Ok(());
    }

    fn call_and_return(&self, ip: u64) -> Result<(), Error> {
        try!(self.vmstate.set_call_return(emu::CODE_SENTINEL));
        return self.vmstate
            .engine
            .borrow()
            .emu_start(ip,
                       emu::CODE_SENTINEL,
                       emu::EMU_TIMEOUT,
                       emu::EMU_MAXCOUNT)
            .map_err(|e| Error::ExecError(e));
    }
}
