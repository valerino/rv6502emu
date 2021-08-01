use crate::bus::Bus;
use crate::memory::Memory;

/**
 * the cpu registers.
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
 * 6502 has 3 vectors (= addresses at which the cpu is directed to perform certain tasks)
 */
#[derive(Debug, PartialEq)]
enum Vectors {
    NMI = 0xfffa,
    RESET = 0xfffc,
    IRQ = 0xfffe,
}

/**
 * implements the cpu.
 */
pub struct Cpu {
    /// cpu registers.
    pub regs: Registers,

    /// current cpu cycles.
    pub cycles: usize,

    /// the bus.
    pub bus: Box<dyn Bus>,
}

impl Cpu {
    /**
     * creates a new cpu instance, with the given Bus attached.
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
     * creates a new cpu instance, with the given Bus attached, exposing a Memory of the given size.
     */
    pub fn new_default(mem_size: usize) -> Cpu {
        let m = super::memory::new_default(mem_size);
        let b = super::bus::new_default(m);
        Cpu::new(b)
    }

    /**
     * resets the cpu setting all registers to the initial values.
     */
    pub fn reset(&mut self) {
        //
        self.regs.pc = Vectors::RESET as u16
    }

    /**
     * run the cpu for the given cycles
     */
    pub fn step(&mut self, cycles: usize) {}

    /**
     * triggers an irq.
     */
    pub fn irq(&mut self) {}

    /**
     * triggers an nmi.
     */
    pub fn nmi(&mut self) {}
}
