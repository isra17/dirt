use emu::Error;
use emu::vmstate::VmState;
use std::cell::RefCell;
use std::rc::Rc;

pub mod linux;

pub trait Env {
    fn attach(&self, vmstate: &mut VmState) -> Rc<RefCell<Kernel>>;
}
pub trait Kernel {
    fn reset(&mut self) -> Result<(), Error>;
    fn detach(&mut self, vmstate: &mut VmState);
}
