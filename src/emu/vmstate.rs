use byteorder::{ByteOrder, LittleEndian};
use emu;
use emu::Error;
use emu::env;
use emu::env::Kernel;
use emu::args::PushableArgs;
use emu::emu_engine::EmuEffects;
use emu::env::Env;
use emu::object_info::{MemMap, ObjectInfo};
use std::cell::RefCell;
use std::rc::Rc;
use unicorn;
use unicorn::unicorn_const::{PROT_EXEC, PROT_READ, PROT_WRITE};
use unicorn::x86_const::RegisterX86 as RegEnum;
use utils::LogError;

pub struct VmState {
    pub engine: Rc<RefCell<unicorn::Unicorn>>,
    pub object_info: ObjectInfo,
    pub stack_info: Option<MemMap>,
    pub emudata_info: Option<MemMap>,
    pub shellcode_info: Option<MemMap>,
    pub snapshot: Vec<(MemMap, Vec<u8>)>,
    pub kernel: Option<Rc<RefCell<Kernel>>>,
}

pub struct DataWriter<'a> {
    write_ptr: u64,
    vmstate: &'a VmState,
}

impl VmState {
    pub fn new(engine: Rc<RefCell<unicorn::Unicorn>>) -> VmState {
        return VmState {
            engine: engine,
            object_info: ObjectInfo::new(),
            stack_info: None,
            emudata_info: None,
            shellcode_info: None,
            snapshot: Default::default(),
            kernel: Default::default(),
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

        // Init emudata.
        let emudata_info = MemMap {
            addr: emu::EMUDATA_ADDR,
            size: emu::EMUDATA_SIZE,
            flags: PROT_READ | PROT_WRITE,
            name: String::from("[emu]"),
        };

        try!(self.mem_map(emudata_info.clone())
            .log_err(|_| String::from("Failed to map emudata")));
        self.emudata_info = Some(emudata_info);

        // Init shellcode memory.
        let shellcode_info = MemMap {
            addr: emu::SHELLCODE_ADDR,
            size: emu::SHELLCODE_SIZE,
            flags: PROT_READ | PROT_WRITE | PROT_EXEC,
            name: String::from("[shellcode]"),
        };

        try!(self.mem_map(shellcode_info.clone())
            .log_err(|_| String::from("Failed to map shellcode")));
        self.shellcode_info = Some(shellcode_info);

        try!(self.init_env());

        try!(self.snapshot());
        return Ok(());
    }

    fn init_env(&mut self) -> Result<(), Error> {
        // TODO: Have a VMEnv trait that set up environment for Linux,
        // Windows, etc.
        let env = env::linux::LinuxEnv {};
        self.kernel = Some(env.attach(self));
        return env::linux::init_state(self);
    }

    pub fn snapshot(&mut self) -> Result<(), Error> {
        self.snapshot.clear();
        for map in self.object_info.mem_maps.values() {
            let mem = try!(self.engine
                .borrow()
                .mem_read(map.addr, map.size));
            self.snapshot.push((map.clone(), mem));
        }
        Ok(())
    }

    pub fn restore_snapshot(&self) -> Result<(), Error> {
        for &(ref map, ref data) in &self.snapshot {
            try!(self.engine.borrow().mem_write(map.addr, data))
        }
        Ok(())
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
            .borrow()
            .reg_read(RegEnum::RSP as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn set_sp(&self, value: u64) -> Result<(), Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .borrow()
            .reg_write(RegEnum::RSP as i32, value)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn ip(&self) -> Result<u64, Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .borrow()
            .reg_read(RegEnum::RIP as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn set_ip(&self, value: u64) -> Result<(), Error> {
        // TODO: Make it arch dependant.
        return self.engine
            .borrow()
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
            try!(self.engine.borrow().mem_write(stack_info.addr, &init_data));
            return Ok(());
        }
        return Err(Error::StackUninitialized);
    }


    pub fn reset_emudata(&self) -> Result<(), Error> {
        if let Some(ref emudata_info) = self.emudata_info {
            let mut init_data: Vec<u8> = Vec::new();
            init_data.resize(emudata_info.size, 0);
            try!(self.engine.borrow().mem_write(emudata_info.addr, &init_data));
            return Ok(());
        }
        return Err(Error::StackUninitialized);
    }


    pub fn return_value(&self) -> Result<u64, Error> {
        return self.engine
            .borrow()
            .reg_read(RegEnum::RAX as i32)
            .map_err(|e| Error::UnicornError(e));
    }

    pub fn stack_push(&self, value: u64) -> Result<(), Error> {
        let sp = try!(self.sp());
        // TODO: Make it arch dependant.
        try!(self.set_sp(sp - 8));
        return self.engine
            .borrow()
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
            vmstate: self,
            return_value: return_value,
            args: args,
        });
    }

    pub fn read_str(&self, addr: u64) -> Result<String, Error> {
        // TODO: Read a page at once, should be faster.
        let mut data_buf: Vec<u8> = vec![];
        let mut i = addr;
        loop {
            match try!(self.engine.borrow().mem_read(i, 1)).pop() {
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
        try!(self.engine.borrow().mem_write(addr, &data_buf));
        return Ok(addr + data_buf.len() as u64);
    }

    pub fn read_usize(&self, addr: u64) -> Result<u64, Error> {
        // TODO: Make it arch independant.
        return Ok(LittleEndian::read_u64(&try!(self.engine
            .borrow()
            .mem_read(addr, 8))));
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

        try!(self.engine
            .borrow()
            .mem_map(mem_map.addr, mem_map.size, mem_map.flags));
        let addr = mem_map.addr;
        self.object_info.mem_maps.insert(mem_map.name.clone(), mem_map);
        return Ok(addr);
    }

    pub fn run_shellcode(&self, code: &[u8]) -> Result<(), Error> {
        let addr = self.shellcode_info.as_ref().unwrap().addr;
        try!(self.engine.borrow().mem_write(addr, code));
        try!(self.engine.borrow().emu_start(addr,
                                            addr + code.len() as u64,
                                            emu::EMU_TIMEOUT,
                                            emu::EMU_MAXCOUNT));

        return Ok(());
    }

    fn native_pack(&self, n: u64) -> Vec<u8> {
        // TODO: Make it arch dependant.
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

    pub fn write_data(&mut self, data: &[u8]) -> Result<u64, Error> {
        let data_ptr = self.write_ptr;
        try!(self.vmstate.engine.borrow().mem_write(self.write_ptr, &data));
        self.write_ptr += data.len() as u64;
        return Ok(data_ptr);
    }

    pub fn write_usize(&mut self, value: u64) -> Result<u64, Error> {
        let data = self.vmstate.native_pack(value);
        return self.write_data(&data);
    }

    pub fn current_ptr(&self) -> u64 {
        return self.write_ptr;
    }
}
