use emu;
use emu::Error;
use emu::args::PushableArgs;
use emu::emu_engine::EmuEffects;
use emu::env;
use emu::object_info::{MemMap, ObjectInfo};
use std::rc::Rc;
use unicorn;
use unicorn::unicorn_const::{PROT_READ, PROT_WRITE};
use unicorn::x86_const::RegisterX86 as RegEnum;
use utils::LogError;

pub struct VmState {
    pub engine: Rc<unicorn::Unicorn>,
    pub object_info: ObjectInfo,
    pub stack_info: Option<MemMap>,
    pub emudata_info: Option<MemMap>,
}

pub struct DataWriter<'a> {
    write_ptr: u64,
    vmstate: &'a VmState,
}

impl VmState {
    pub fn new(engine: Rc<unicorn::Unicorn>) -> VmState {
        return VmState {
            engine: engine,
            object_info: ObjectInfo::new(),
            stack_info: None,
            emudata_info: None,
        };
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // Init the stack.
        let stack_info = MemMap {
            addr: emu::STACK_ADDR,
            size: emu::STACK_SIZE,
            flags: PROT_READ | PROT_WRITE,
            name: String::from("[stack]"),
        };

        try!(self.mem_map(stack_info.clone())
            .log_err(|_| String::from("Failed to map stack")));
        self.stack_info = Some(stack_info);
        self.set_sp(self.base_sp().unwrap())
            .expect("Failed to set sp to base of stack");

        let emudata_info = MemMap {
            addr: emu::EMUDATA_ADDR,
            size: emu::EMUDATA_SIZE,
            flags: PROT_READ | PROT_WRITE,
            name: String::from("[emu]"),
        };

        try!(self.mem_map(emudata_info.clone())
            .log_err(|_| String::from("Failed to map emudata")));
        self.emudata_info = Some(emudata_info);

        // try!(self.init_env());

        return Ok(());
    }

    fn init_env(&mut self) -> Result<(), Error> {
        // TODO: Have a VMEnv trait that set up environment for Linux,
        // Windows, etc.
        return env::linux::init_state(self);
    }

    pub fn emudata_writer<'a>(&'a self) -> Result<DataWriter<'a>, Error> {
        if let Some(ref emudata) = self.emudata_info {
            return Ok(DataWriter::new(self, emudata.addr));
        }
        return Err(Error::EmuDataUninitialized);
    }

    pub fn base_sp(&self) -> Option<u64> {
        return match self.stack_info {
            Some(ref s) => Some(s.addr + s.size as u64),
            None => None,
        };
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

    pub fn reset_stack(&self) -> Result<(), Error> {
        if let Some(ref stack_info) = self.stack_info {
            let base_sp = self.base_sp().unwrap();
            try!(self.set_sp(base_sp)
                .log_err(|_| {
                    String::from("Failed to set sp to base of stack")
                }));
            let mut init_data: Vec<u8> = Vec::new();
            init_data.resize(stack_info.size, 0);
            try!(self.engine.mem_write(stack_info.addr, &init_data));
            return Ok(());
        }
        return Err(Error::StackUninitialized);
    }


    pub fn reset_emudata(&self) -> Result<(), Error> {
        if let Some(ref emudata_info) = self.emudata_info {
            let mut init_data: Vec<u8> = Vec::new();
            init_data.resize(emudata_info.size, 0);
            try!(self.engine.mem_write(emudata_info.addr, &init_data));
            return Ok(());
        }
        return Err(Error::StackUninitialized);
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
        // TODO: Read a page at once, should be faster.
        let mut data_buf: Vec<u8> = vec![];
        let mut i = addr;
        loop {
            match try!(self.engine.mem_read(i, 1)).pop() {
                None => break,
                Some(0) => break,
                Some(b) => data_buf.push(b),
            }

            i += 1;
        }
        return String::from_utf8(data_buf).map_err(|e| Error::FromUtf8Error(e));
    }

    pub fn write_str(&self, addr: u64, data: &str) -> Result<u64, Error> {
        let mut data_buf = data.as_bytes().to_vec();
        data_buf.push(0);
        try!(self.engine.mem_write(addr, &data_buf));
        return Ok(addr + data_buf.len() as u64);
    }

    /// Unlike unicorn.mem_map, this function keep track of the mapping
    /// and provide a reverse function to find mapping given a name.
    /// The mapping address and size must still be aligned.
    pub fn mem_map(&mut self, mut mem_map: MemMap) -> Result<u64, Error> {
        if mem_map.name.is_empty() {
            mem_map.name = format!("anon:{:x}", mem_map.addr)
        }

        if self.object_info.mem_maps.contains_key(&mem_map.name) {
            return Err(Error::MapAlreadyExists);
        }

        try!(self.engine.mem_map(mem_map.addr, mem_map.size, mem_map.flags));
        let addr = mem_map.addr;
        self.object_info.mem_maps.insert(mem_map.name.clone(), mem_map);
        return Ok(addr);
    }

    fn native_pack(&self, n: u64) -> Vec<u8> {
        // TODO: Make it arch dependant.
        use byteorder::{ByteOrder, LittleEndian};
        let mut packed = [0; 8];
        LittleEndian::write_u64(&mut packed, n);
        return packed.to_vec();
    }
}

impl<'a> DataWriter<'a> {
    pub fn new(vmstate: &'a VmState, write_ptr: u64) -> DataWriter<'a> {
        return DataWriter {
            write_ptr: write_ptr,
            vmstate: vmstate,
        };
    }

    pub fn write_str(&mut self, data: &str) -> Result<u64, Error> {
        let str_ptr = self.write_ptr;
        self.write_ptr = try!(self.vmstate.write_str(self.write_ptr, data));
        return Ok(str_ptr);
    }
}
