use crate::bus::Bus;

/**
 * the cpu registers
 */
pub struct Registers {
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,
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
    bus: Box<dyn Bus>,
}

impl Cpu {
    /**
     * creates a new cpu instance
     */
    pub fn new(b: Box<dyn Bus>) -> Cpu {
        let c = Cpu {
            regs: Registers::new(),
            cycles: 0,
            bus: b,
        };
        c
    }

    /**
     * gets the bus
     */
    pub fn get_bus(&mut self) -> &mut Box<dyn Bus> {
        &mut self.bus
    }

    /**
     * gets the current cpu cycles
     */
    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    /**
     * gets the registers
     */
    pub fn get_registers(&self) -> &Registers {
        &self.regs
    }
}
