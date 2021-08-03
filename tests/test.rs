use log::*;
use rv6502emu::bus;
use rv6502emu::cpu::Cpu;
use rv6502emu::gui::DebuggerUi;
use rv6502emu::memory;
use rv6502emu::memory::memory_error::MemoryError;
use rv6502emu::memory::Memory;
use std::prelude::*;
use std::sync::Arc;

fn test_inner(mem: &mut Box<dyn Memory>) {
    let b = mem.read_byte(123).unwrap();
    info!("b after write 2={:x}", b);
    assert_eq!(b, 0xfc);
}

fn test_read_writes(mem: &mut Box<dyn Memory>) {
    // some read and writes
    let mut bb = mem.read_byte(123).unwrap();
    info!("b after read ={}", bb);
    assert_eq!(bb, 0xff);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(123).unwrap();
    info!("b after write 1={:x}", bb);
    assert_eq!(bb, 0xaa);

    // some read and writes in a function
    mem.write_byte(123, 0xfc);
    test_inner(mem);

    let b = mem.read_byte(123).unwrap();
    assert_eq!(b, 0xfc)
}

/**
 * tests the cpu using klaus test (https://github.com/Klaus2m5/6502_65C02_functional_tests)
 */
#[test]
fn test_cpu() {
    // init logger
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .try_init();

    // run the cpu thread
    let mut dbg_ui = rv6502emu::gui::new();
    dbg_ui.start_comm_thread();
    info!("running gui");
    dbg_ui.run();
    // create a cpu with default bus and 64k memory
    let mut c = rv6502emu::cpu::Cpu::new_default(0x10000);
    let mem = c.bus.get_memory();
    // test_read_writes(mem);

    // load test file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();

    // resets the cpu and start execution
    c.reset();
    info!("cpu thread handle={:?}", std::thread::current());
    let mut dbg_ui = rv6502emu::gui::new();
    dbg_ui.start_comm_thread();
    info!("running gui");
    dbg_ui.run();

    /*
    //let mut dbg_ui = rv6502emu::gui::new(&c.to_ui_channels, &c.from_ui_channels);
    //let t_handle = dbg_ui.run();
    info!("receiving.....");
    c.from_ui_channels.1.recv();
    info!("received!");
    t_handle.join();
    */
    /*
    // some read and writes
    let mut bb = mem.read_byte(123).unwrap();
    info!("b after read ={}", bb);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(123).unwrap();
    info!("b after write={:x}", bb);

    // some read and writes in a function
    tt(mem);
    mem.write_byte(123, 0xfc);
    tt(mem)
    */

    // run test
    /*
    loop {
        c.step(0);
    }*/
}
