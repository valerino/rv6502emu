use crate::bus::Bus;
use crate::gui::{DebuggerUi, UiContext};
use crate::memory::memory_error::MemoryError;
use crate::memory::Memory;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Error, Formatter};
use std::prelude::*;

/**
 * the cpu registers.
 */
#[derive(Debug, Serialize, Deserialize)]
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

    /// the bus.
    pub bus: Box<dyn Bus>,

    /// channels for communications TO the ui.
    pub to_ui_channels: (Sender<UiContext>, Receiver<UiContext>),

    /// channels for communications FROM the ui.
    pub from_ui_channels: (Sender<UiContext>, Receiver<UiContext>),
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
            to_ui_channels: unbounded::<UiContext>(),
            from_ui_channels: unbounded::<UiContext>(),
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
    pub fn reset(&mut self) -> Result<(), MemoryError> {
        // from https://www.pagetable.com/?p=410
        self.regs = Registers {
            a: 0xaa,
            x: 0,
            y: 0,
            p: 0x16,
            s: 0xfd,

            // at reset, we read PC from RESET vector
            pc: self
                .bus
                .get_memory()
                .read_word_le(Vectors::RESET as usize)?,
        };
        println!("{}", self.regs);
        Ok(())
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
