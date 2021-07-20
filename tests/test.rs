use rv6502emu::cpu::Cpu;
use rv6502emu::memory;
use rv6502emu::memory::Memory;

fn tt(mem: &mut Memory) {
    mem.write_byte(123, 0xbb);
    let b = mem.read_byte(123).unwrap();
    println!("b after write 2={:x}", b);
}

#[test]
fn test_cpu() {
    let mut m = memory::new(1234);
    let mut c = Cpu::new(&mut m);

    let c_mem = c.mm();
    let mut b = c_mem.read_byte(123).unwrap();
    println!("b after read ={}", b);
    c_mem.write_byte(123, 0xaa);

    b = c_mem.read_byte(123).unwrap();
    println!("b after write={:x}", b);
    tt(c_mem)
}
