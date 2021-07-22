use rv6502emu::bus;
use rv6502emu::bus::Bus;
use rv6502emu::cpu::Cpu;
use rv6502emu::memory;
use rv6502emu::memory::Memory;

fn tt(mem: &mut Memory) {
    let b = mem.read_byte(123).unwrap();
    println!("b after write 2={:x}", b);
}

#[test]
fn test_cpu() {
    // create a new memory and a bus with it attached
    let mut m = memory::new(1234);
    let mut b = bus::new(Box::new(m));

    // creates a new cpu with the given bus
    let bb = Box::new(b);
    let c = Cpu::new(&bb);

    // some read and writes
    let mem = b.memory();
    let mut b = mem.read_byte(123).unwrap();
    println!("b after read ={}", b);
    mem.write_byte(123, 0xaa);

    // read again
    b = mem.read_byte(123).unwrap();
    println!("b after write={:x}", b);

    // some read and writes in a function
    tt(mem);
    mem.write_byte(123, 0xfc);
    tt(mem)
}
