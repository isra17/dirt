use dirt_engine::TargetInfo;
use emu::Error;
use emu::args::{EmuArgs, PushableArgs};
use emu::debugger::Debugger;
use emu::vmstate::VmState;

pub struct EmuEffects {
    pub return_value: u64,
    pub args: PushableArgs,
}

pub struct EmuEngine {
    pub vmstate: VmState,
    pub debugger: Option<Debugger>,
}

const CODE_SENTINEL: u64 = 0x80000000;
const EMU_TIMEOUT: u64 = 1 * 1000 * 1000; // 1 sec.
const EMU_MAXCOUNT: usize = 0;

impl EmuEngine {
    pub fn new(mut vmstate: VmState) -> Result<EmuEngine, Error> {
        // Code sentinel used to trap function return.
        vmstate.engine
            .mem_map(CODE_SENTINEL,
                     0x1000,
                     ::unicorn::unicorn_const::PROT_EXEC)
            .expect("Failed to map code sentinel");
        try!(vmstate.init());
        let debugger = try!(Debugger::attach(vmstate.engine.clone()));

        return Ok(EmuEngine {
            vmstate: vmstate,
            debugger: Some(debugger),
        });
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
        println!("Calling 0x{:016x}", ip);
        try!(self.vmstate.set_call_return(CODE_SENTINEL));
        return self.vmstate
            .engine
            .emu_start(ip, CODE_SENTINEL, EMU_TIMEOUT, EMU_MAXCOUNT)
            .map_err(|e| Error::ExecError(e));
    }
}
