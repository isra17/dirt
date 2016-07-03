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

    // Emulate up to main.
    let start_fva = vmstate.object_info
        .symbols
        .get("_start")
        .expect("_start not found")
        .value;
    let main_fva =
        vmstate.object_info.symbols.get("main").expect("main not found").value;
    try!(vmstate.engine
        .emu_start(start_fva, main_fva, emu::EMU_TIMEOUT, emu::EMU_MAXCOUNT));

    return Ok(());
}

fn init_stack(vmstate: &VmState,
              data_writer: &mut DataWriter)
              -> Result<(), Error> {
    // auxv
    try!(vmstate.stack_push(0));
    // env
    try!(vmstate.stack_push(0));
    // argv
    try!(vmstate.stack_push(0));
    try!(vmstate.stack_push(try!(data_writer.write_str("./emu"))));

    // argc
    try!(vmstate.stack_push(1));

    return Ok(());
}
