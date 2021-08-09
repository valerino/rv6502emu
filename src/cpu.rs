use crate::bus::Bus;
use crate::memory::memory_error::MemoryError;
use crate::memory::Memory;
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

/**
 * indicates the operation CpuCallbackContext refers to.
 */
#[derive(Debug, PartialEq)]
pub enum CpuOperation {
    Read,
    Write,
    Irq,
    Nmi,
}

/**
 * this is passed by the cpu to the caller when reads/writes/irq/nmi occurs.
 */
#[derive(Debug)]
pub struct CpuCallbackContext {
    pub address: usize,
    pub value: u8,
    pub operation: CpuOperation,
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

    // debugger enabled/disabled.
    pub debug: bool,

    /// the bus.
    pub bus: Box<dyn Bus>,

    // callback for the caller.
    pub cb: fn(cb: CpuCallbackContext),
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
    pub fn new(b: Box<dyn Bus>, cb: fn(cb: CpuCallbackContext), debug: bool) -> Cpu {
        let c = Cpu {
            regs: Registers::new(),
            cycles: 0,
            bus: b,
            debug: debug,
            cb: cb,
        };
        c
    }

    /**
     * creates a new cpu instance, with the given Bus attached, exposing a Memory of the given size.
     */
    pub fn new_default(mem_size: usize, cb: fn(cb: CpuCallbackContext), debug: bool) -> Cpu {
        let m = super::memory::new_default(mem_size);
        let b = super::bus::new_default(m);
        Cpu::new(b, cb, debug)
    }

    /**
     * resets the cpu setting all registers to the initial values.
     */
    pub fn reset(&mut self, start_address: Option<u16>) -> Result<(), MemoryError> {
        let mut addr: u16 = 0;
        if let Some(a) = start_address {
            // use provided address
            addr = a;
        } else {
            // get the start address from reset vector
            // from https://www.pagetable.com/?p=410
            addr = self
                .bus
                .get_memory()
                .read_word_le(Vectors::RESET as usize)?;
        }
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
        Ok(())
    }

    /**
     * fetch opcode at PC
     */
    fn fetch(&mut self) -> Result<u8, MemoryError> {
        let mem = self.bus.get_memory();
        let b = mem.read_byte(self.regs.pc as usize)?;
        Ok(b)
    }

    /**
     * run the cpu for the given cycles, pass 0 to run indefinitely.
     *
     * > note that reset() must be called first to set the start address !
     */
    pub fn run(&mut self, cycles: usize) -> Result<(), MemoryError> {
        loop {
            // fetch an instruction
            let b = self.fetch()?;

            // decode and run
            let (opcode_f, opcode_cycles) = opcodes::OPCODE_MATRIX[b as usize];
            let _ = match opcode_f(self, opcode_cycles) {
                Ok(elapsed) => {
                    // advance pc and increment the elapsed cycles
                    self.regs.pc += 1;
                    self.cycles += elapsed;
                    if cycles != 0 && elapsed >= cycles {
                        break;
                    }
                }
                // panic here ...
                // TODO: break on debug
                Err(e) => return Err(e),
            };
        }
        Ok(())
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
