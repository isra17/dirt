use std::rc::Rc;
use unicorn;

pub struct VmState {
    uc: Rc<unicorn::CpuX86>,
}

pub struct EmuData<'a> {
    write_ptr: u64,
    vmstate: &'a VmState,
}


impl VmState {
    pub fn new(uc: Rc<unicorn::CpuX86>) -> VmState {
        return VmState { uc: uc };
    }

    pub fn usr_data<'a>(&'a self) -> EmuData<'a> {
        return EmuData::new(self);
    }
}

impl<'a> EmuData<'a> {
    pub fn new(vmstate: &'a VmState) -> EmuData<'a> {
        return EmuData {
            write_ptr: 0, // vmstate.segment_ptr("[emu]"),
            vmstate: vmstate,
        };
    }

    pub fn write_str(&self, data: &str) -> u64 {
        return self.write_ptr;
    }
}
