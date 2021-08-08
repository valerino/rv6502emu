use crate::bus::Bus;
use log::*;
mod opcodes;
use opcodes::*;
use std::fmt::{Display, Error, Formatter};
mod addressing_modes;

/**
 * the cpu registers.
 */
#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,
}

impl Display for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PC: ${:02x}, A: ${:x}, X: ${:x}, Y: ${:x}, P: {:08b}, S: ${:x}",
            self.pc, self.a, self.x, self.y, self.p, self.s
        );
        Ok(())
    }
}

impl Registers {
    pub fn new() -> Registers {
        let r = Registers {
            a: 0,
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

    // debugger enabled/disabled
    pub debug: bool,

    /// the bus.
    pub bus: Box<dyn Bus>,
}

impl Cpu {
    /**
     * activate logging on stdout trough env_logger (max level)
     */
    pub fn enable_logging(enable: bool) {
        if enable == true {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::max())
                .try_init();
        }
    }

    /**
     * creates a new cpu instance, with the given Bus attached.
     */
    pub fn new(b: Box<dyn Bus>, debug: bool) -> Cpu {
        let c = Cpu {
            regs: Registers::new(),
            cycles: 0,
            bus: b,
            debug: debug,
        };
        c
    }

    /**
     * creates a new cpu instance, with the given Bus attached, exposing a Memory of the given size.
     */
    pub fn new_default(mem_size: usize, debug: bool) -> Cpu {
        let m = super::memory::new_default(mem_size);
        let b = super::bus::new_default(m);
        Cpu::new(b, debug)
    }

    /**
     * resets the cpu setting all registers to the initial values.
     */
    pub fn reset(&mut self) {
        // get the start address from reset vector
        // from https://www.pagetable.com/?p=410
        let addr = self
            .bus
            .get_memory()
            .read_word_le(Vectors::RESET as usize)
            .unwrap();

        self.regs = Registers {
            a: 0xaa,
            x: 0,
            y: 0,
            p: 0x16,
            s: 0xfd,

            // at reset, we read PC from RESET vector
            pc: addr,
        };
        debug!("RESET\n--> {}", self.regs);
        opcodes::OPCODE_MATRIX[1](self);
        opcodes::OPCODE_MATRIX[0](self);
    }

    /**
     * step an instruction and return elapsed cycles
     */
    fn step(&mut self) -> usize {
        let op = opcodes::fetch(&self);
        0
    }

    /**
     * run the cpu for the given cycles, pass 0 to run indefinitely
     */
    pub fn run(&mut self, cycles: usize) {
        loop {}
    }

    /**
     * triggers an irq.
     */
    pub fn irq(&mut self) {}

    /**
     * triggers an nmi.
     */
    pub fn nmi(&mut self) {}
}
