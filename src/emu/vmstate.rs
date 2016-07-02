use emu::Error;
use emu::args::PushableArgs;
use emu::emu_engine::EmuEffects;
use std::rc::Rc;
use unicorn;
use unicorn::x86_const::RegisterX86 as RegEnum;

pub struct VmState {
    pub engine: Rc<unicorn::Unicorn>,
}

pub struct DataWriter<'a> {
    write_ptr: u64,
    vmstate: &'a VmState,
}

impl VmState {
    pub fn new(engine: Rc<unicorn::Unicorn>) -> VmState {
        return VmState { engine: engine };
    }

    pub fn emu_data<'a>(&'a self) -> DataWriter<'a> {
        return DataWriter::new(self);
    }

    pub fn sp(&self) -> Result<u64, Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .reg_read(RegEnum::RSP as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn set_sp(&self, value: u64) -> Result<(), Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .reg_write(RegEnum::RSP as i32, value)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn ip(&self) -> Result<u64, Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .reg_read(RegEnum::RIP as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn set_ip(&self, value: u64) -> Result<(), Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .reg_write(RegEnum::RIP as i32, value)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn return_value(&self) -> Result<u64, Error> {
        return self.engine
            .reg_read(RegEnum::RAX as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn stack_push(&self, value: u64) -> Result<(), Error> {
        let sp = try!(self.sp());
        // TODO: Make it arch dependant.
        try!(self.set_sp(sp - 8));
        return self.engine
            .mem_write(sp - 8, &self.native_pack(value))
            .map_err(|e| Error::UnicornError(e));
    }


    /// Set the emulator state's return value.
    pub fn set_call_return(&self, return_va: u64) -> Result<(), Error> {
        // TODO: Need to make arch dependant?
        try!(self.stack_push(return_va));
        return Ok(());
    }

    pub fn collect_call_results(&self,
                                args: PushableArgs)
                                -> Result<EmuEffects, Error> {
        let return_value = try!(self.return_value());
        return Ok(EmuEffects {
            return_value: return_value,
            args: args,
        });
    }

    pub fn read_str(&self, addr: u64) -> Result<String, Error> {
        return Err(Error::NotImplemented);
    }

    pub fn write_str(&self, addr: u64, data: &str) -> Result<u64, Error> {
        return Err(Error::NotImplemented);
    }

    fn native_pack(&self, n: u64) -> Vec<u8> {
        // TODO: Make it arch dependant.
        use byteorder::{ByteOrder, LittleEndian};
        let mut packed = Vec::with_capacity(8);
        LittleEndian::write_u64(packed.as_mut_slice(), n);
        return packed;
    }
}

impl<'a> DataWriter<'a> {
    pub fn new(vmstate: &'a VmState) -> DataWriter<'a> {
        return DataWriter {
            write_ptr: 0, // vmstate.segment_ptr("[emu]"),
            vmstate: vmstate,
        };
    }

    pub fn write_str(&mut self, data: &str) -> Result<u64, Error> {
        let str_ptr = self.write_ptr;
        self.write_ptr = try!(self.vmstate.write_str(self.write_ptr, data));
        return Ok(str_ptr);
    }
}
