use rv6502emu::bus;
use rv6502emu::cpu::Cpu;
use rv6502emu::memory;
use rv6502emu::memory::Memory;

fn tt(mem: &Box<dyn Memory>) {
    let b = mem.read_byte(123).unwrap();
    println!("b after write 2={:x}", b);
}

#[test]
fn test_cpu() {
    // create a new memory and a bus with it attached
    let m = memory::new(1234);
    let b = bus::new(m);

    // creates a new cpu with the given bus
    let mut c = Cpu::new(b);

    // some read and writes
    let mem = c.get_bus().get_memory();
    let mut bb = mem.read_byte(123).unwrap();
    println!("b after read ={}", bb);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(44444123).unwrap();
    println!("b after write={:x}", bb);

    // some read and writes in a function
    tt(mem);
    mem.write_byte(123, 0xfc);
    tt(mem)
}
