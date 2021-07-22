use rv6502emu::bus;
use rv6502emu::bus::Bus;
use rv6502emu::cpu::Cpu;
use rv6502emu::memory;
use rv6502emu::memory::Memory;

fn tt(mem: &mut Box<dyn Memory>) {
    let b = mem.read_byte(123).unwrap();
    println!("b after write 2={:x}", b);
}

#[test]
fn test_cpu() {
    // create a new memory and a bus with it attached
    let m = Box::new(memory::new(1234));
    let b = bus::new(m);

    // creates a new cpu with the given bus
    let mut c = Cpu::new(b);

    // some read and writes
    let mut mem = c.bus().memory();
    let mut b = mem.read_byte(123).unwrap();
    println!("b after read ={}", b);
    mem.write_byte(123, 0xaa);

    // read again
    b = mem.read_byte(123).unwrap();
    println!("b after write={:x}", b);

    // some read and writes in a function
    tt(&mut mem);
    mem.write_byte(123, 0xfc);
    tt(&mut mem)
}
