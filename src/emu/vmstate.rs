use std::rc::Rc;
use unicorn;
use emu::Error;
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

    pub fn stack_push(&self, value: u64) -> Result<(), Error> {
        let sp = try!(self.sp());
        // TODO: Make it arch dependant.
        try!(self.set_sp(sp - 8));
        return self.engine
            .mem_write(sp - 8, &self.native_pack(value))
            .map_err(|e| Error::UnicornError(e));
    }

    /// Set the vm to a state right before calling |fva| and with the return
    /// location set to |return_va|.
    pub fn set_call(&self, fva: u64, return_va: u64) -> Result<(), Error> {
        // TODO: Need to make arch dependant?
        try!(self.stack_push(return_va));
        try!(self.set_ip(fva));
        return Ok(());
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

    pub fn write_str(&self, data: &str) -> u64 {
        return self.write_ptr;
    }
}
