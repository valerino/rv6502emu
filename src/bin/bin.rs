use rv6502emu::cpu::debugger::Debugger;
use rv6502emu::cpu::Cpu;
use rv6502emu::cpu::CpuCallbackContext;

fn test_callback(_c: &mut Cpu, _cb: CpuCallbackContext) {
    //info!("{}", cb);
}

pub fn main() {
    // create a cpu with default bus and 64k memory
    let mut c = Cpu::new_default(0x10000, Some(test_callback));

    // enable stdout logger
    c.enable_logging(true);

    let mem = c.bus.get_memory();

    // load test file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();

    // resets the cpu (use 0x400 as custom address for the Klaus test) and start execution
    c.reset(Some(0x400)).unwrap();

    // run with a debugger attached, setting an r/w breakpoint before starting
    let mut dbg = Debugger::new(true);
    dbg.parse_cmd(&mut c, "bw $200");

    // run !
    c.run(Some(&mut dbg), 0).unwrap();
    // or, run without debugger attached
    //c.run(None, 0).unwrap();
}
