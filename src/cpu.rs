use crate::bus::Bus;

/**
 * the cpu registers
 */
struct Registers {
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    pc: u8,
}

impl Registers {
    fn new() -> Registers {
        let r = Registers {
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc: 0,
        };
        r
    }
}

/**
 * implements the cpu.
 */
pub struct Cpu {
    regs: Registers,
    cycles: usize,
    //b: &'a dyn Bus,
    b: Box<dyn Bus>,
}

impl Cpu {
    /**
     * creates a new cpu instance
     */
    pub fn new(b: Box<dyn Bus>) -> Cpu {
        let c = Cpu {
            regs: Registers::new(),
            cycles: 0,
            b: b,
        };
        c
    }

    /**
     * gets the underlying bus
     */
    pub fn bus(&mut self) -> &mut Box<dyn Bus> {
        &mut self.b
    }
}
