use crate::memory::Memory;

/**
 * implements the cpu.
 */
pub struct Cpu<'a> {
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    pc: u8,
    cycles: usize,
    pub mem: &'a mut (dyn Memory),
}

impl<'a> Cpu<'a> {
    pub fn mm(&'a mut self) -> &'a mut (dyn Memory) {
        self.mem
    }

    pub fn new(m: &'a mut dyn Memory) -> Cpu<'a> {
        let c = Cpu {
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc: 0,
            cycles: 0,
            mem: m,
        };
        c
    }
}
