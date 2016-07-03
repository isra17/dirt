use emu;
use emu::Error;
use emu::object_info::MemMap;
use emu::vmstate::{DataWriter, VmState};
use unicorn::unicorn_const::{PROT_READ, PROT_WRITE};

pub fn init_state(vmstate: &mut VmState) -> Result<(), Error> {
    // Set up the program stack as it would look from the kernel and
    // emulate <__start> up to <main>. This should give us a nice
    // initialized program state, if it worked...

    let kernel_map_addr = try!(vmstate.mem_map(MemMap {
        addr: emu::KERNEL_ADDR,
        size: emu::KERNEL_SIZE,
        flags: PROT_READ | PROT_WRITE,
        name: String::from("[kernel]"),
    }));
    let mut kernel_writer = DataWriter::new(vmstate, kernel_map_addr);

    try!(init_stack(vmstate, &mut kernel_writer));

    return Ok(());
}

fn init_stack(vmstate: &VmState,
              data_writer: &mut DataWriter)
              -> Result<(), Error> {
    try!(vmstate.stack_push(try!(data_writer.write_str(""))));
    return Ok(());
}
