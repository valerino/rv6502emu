use rv6502emu::bus;
use rv6502emu::cpu::Cpu;
use rv6502emu::memory;
use rv6502emu::memory::memory_error::MemoryError;
use rv6502emu::memory::Memory;

fn tt(mem: &Box<dyn Memory>) {
    let b = mem.read_byte(123).unwrap();
    println!("b after write 2={:x}", b);
}

#[test]
fn test_cpu() {
    // create a new memory and a bus with it attached
    let m = memory::new_default(65536);
    let b = bus::new_default(m);

    // creates a new cpu with the given bus
    let mut c = Cpu::new(b);

    let mem = c.bus.get_memory();

    // load file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();
    // some read and writes
    let mut bb = mem.read_byte(123).unwrap();
    println!("b after read ={}", bb);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(123).unwrap();
    println!("b after write={:x}", bb);

    // some read and writes in a function
    tt(mem);
    mem.write_byte(123, 0xfc);
    tt(mem)
}
