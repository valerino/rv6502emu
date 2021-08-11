/*
 * Filename: /src/cpu.rs
 * Project: rv6502emu
 * Created Date: 2021-08-09, 12:51:43
 * Author: valerino <xoanino@gmail.com>
 * Copyright (c) 2021 valerino
 *
 * MIT License
 *
 * Copyright (c) 2021 valerino
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::bus::Bus;
use log::*;
mod opcodes;
use std::fmt::{Display, Error, Formatter};

use bitflags::bitflags;
pub(crate) mod addressing_modes;

pub mod cpu_error;
use cpu_error::{CpuError, CpuErrorType};

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

bitflags! {
    /**
     * flags (values for the P register).
     * https://www.atarimagazines.com/compute/issue53/047_1_All_About_The_Status_Register.php
     */
    pub(crate) struct CpuFlags : u8 {
        /**
         * C (bit 0)—Carry flag. Carry is set whenever the accumulator rolls over from $FF to $00.
         */
        const C = 0b00000001;
        /**
         * Z (bit 1)—Zero flag. This one's used a great deal, and basically the computer sets it when the result of any operation is zero, i.e. Load the X-register with $00, and you set the zero flag,
         * subtract $32 from $32 and you set the zero flag, ...
         */
        const Z = 0b00000010;
        /**
         * I (bit 2)—Interrupt mask. When this bit is set, the computer will not honor interrupts, such as those used for keyboard scanning in many computers.
         */
        const I = 0b00000100;
        /**
         * D (bit 3)—Decimal flag. When D is set by the programmer, the 6502 does its arithmetic in BCD, binary coded decimal, which is yet another exotic type of computer math.
         */
        const D = 0b00001000;
        /**
         * B (bit 4)—Break flag, set whenever a BRK instruction is executed, clear at all other times.
         */
        const B = 0b00010000;
        /**
         * Bit 5 has no name, and is always set to 1.
         */
        const U = 0b00100000;
        /**
         * V (bit 6)—Overflow flag. This flag is important in twos complement arithmetic, but elsewhere it is rarely used.
         */
        const V = 0b01000000;
        /**
         * N (bit 7)—Negative flag. (Some books call it S, for sign.) The N flag matches the high bit of the result of whatever operation the processor has just completed.
         * If you load $FF (1111 1111) into the Y-register, for example, since the high bit of the Y-register is set, the N flag will be set, too.
         * ML programmers make good use of the N flag. (By the way, even though this is the eighth bit, we call it bit 7, because computers start numbering things at 0.)
         * In a computer technique called twos complement arithmetic, the high-order bit of a number is set to 1 if the number is negative, and cleared to 0 if it's positive,
         * and that's where the N flag gets its name.
         */
        const N = 0b10000000;
    }
}

/**
 * this is called by the cpu to provide the user with notification when reads/writes/irq/nmi occurs.
 */
#[derive(Debug)]
pub struct CpuCallbackContext {
    pub address: u16,
    pub value: u8,
    pub operation: CpuOperation,
}

impl Display for CpuCallbackContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "CALLBACK! type={:?}, address=${:04x}, value=${:02x}",
            self.operation, self.address, self.value
        );
        Ok(())
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PC: ${:04x}, A: ${:02x}, X: ${:02x}, Y: ${:02x}, P: {:08b}({}), S: ${:02x}",
            self.pc,
            self.a,
            self.x,
            self.y,
            self.p,
            self.flags_to_string(),
            self.s
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

    /**
     * convert P (flags) register to a meaningful string
     */
    fn flags_to_string(&self) -> String {
        let p = CpuFlags::from_bits(self.p).unwrap();
        let s = format!(
            "{}{}{}{}{}{}{}{}",
            if p.contains(CpuFlags::N) { "N" } else { "-" },
            if p.contains(CpuFlags::V) { "V" } else { "-" },
            if p.contains(CpuFlags::U) { "U" } else { "-" },
            if p.contains(CpuFlags::B) { "B" } else { "-" },
            if p.contains(CpuFlags::D) { "D" } else { "-" },
            if p.contains(CpuFlags::I) { "I" } else { "-" },
            if p.contains(CpuFlags::Z) { "Z" } else { "-" },
            if p.contains(CpuFlags::C) { "C" } else { "-" },
        );
        s
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

    // callback for the user (optional).
    pub cb: Option<fn(cb: CpuCallbackContext)>,
}

impl Cpu {
    /**
     * call installed cpu callback if any
     */
    pub(crate) fn call_callback(&self, address: u16, value: u8, op: CpuOperation) {
        if self.cb.is_some() {
            // call callback
            let ctx = CpuCallbackContext {
                address: address,
                value: value,
                operation: op,
            };
            self.cb.unwrap()(ctx);
        }
    }

    /**
     * set/unset the carry flag
     */
    pub(crate) fn set_carry_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::C, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the negative flag
     */
    pub(crate) fn set_negative_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::N, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the overflow flag
     */
    pub(crate) fn set_overflow_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::V, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the zero flag
     */
    pub(crate) fn set_zero_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::Z, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the decimal flag
     */
    pub(crate) fn set_decimal_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::D, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the interrupt disable flag
     */
    pub(crate) fn set_interrupt_disable_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::I, set);
        self.regs.p = p.bits();
    }

    /**
     * set/unset the break flag
     */
    pub(crate) fn set_break_flag(&mut self, set: bool) {
        let mut p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.set(CpuFlags::B, set);
        self.regs.p = p.bits();
    }

    /**
     * check if the D flag is set
     */
    pub(crate) fn is_decimal_set(&self) -> bool {
        let p = CpuFlags::from_bits(self.regs.p).unwrap();
        p.contains(CpuFlags::D)
    }

    /**
     * returns 1 if the carry flag is set
     */
    pub(crate) fn is_carry_set(&self) -> u8 {
        let p = CpuFlags::from_bits(self.regs.p).unwrap();
        if p.contains(CpuFlags::C) {
            return 1;
        }
        0
    }

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
    pub fn new(b: Box<dyn Bus>, cb: Option<fn(cb: CpuCallbackContext)>, debug: bool) -> Cpu {
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
    pub fn new_default(
        mem_size: usize,
        cb: Option<fn(cb: CpuCallbackContext)>,
        debug: bool,
    ) -> Cpu {
        let m = super::memory::new_default(mem_size);
        let b = super::bus::new_default(m);
        Cpu::new(b, cb, debug)
    }

    /**
     * resets the cpu setting all registers to the initial values.
     * http://forum.6502.org/viewtopic.php?p=2959
     */
    pub fn reset(&mut self, start_address: Option<u16>) -> Result<(), CpuError> {
        let addr: u16;
        if let Some(a) = start_address {
            // use the provided address
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
            a: 0,
            x: 0,
            y: 0,

            // I (enable interrupts), and the U flag is always set.
            p: (CpuFlags::U | CpuFlags::I).bits(),
            s: 0xff,

            // at reset, we read PC from RESET vector
            pc: addr,
        };
        debug!("RESET!");
        Ok(())
    }

    /**
     * fetch opcode at PC
     */
    fn fetch(&mut self) -> Result<u8, CpuError> {
        let mem = self.bus.get_memory();
        let b = mem.read_byte(self.regs.pc as usize)?;
        Ok(b)
    }

    /**
     * run the cpu for the given cycles, pass 0 to run indefinitely.
     *
     * > note that reset() must be called first to set the start address !
     */
    pub fn run(&mut self, cycles: usize) -> Result<(), CpuError> {
        loop {
            // fetch an instruction
            let b = self.fetch()?;

            // decode
            let (opcode_f, opcode_cycles, add_extra_cycle_on_page_crossing) =
                opcodes::OPCODE_MATRIX[b as usize];

            // print registers
            self.debug_out_registers();

            // execute
            let (instr_size, elapsed) =
                opcode_f(self, opcode_cycles, add_extra_cycle_on_page_crossing).unwrap_or_else(
                    |e| {
                        // error !
                        // TODO: handle with debugger if attached
                        debug!("{}", e);
                        panic!();
                    },
                );

            // advance pc and increment the elapsed cycles
            self.regs.pc += instr_size as u16;
            self.cycles += elapsed;
            if cycles != 0 && elapsed >= cycles {
                break;
            }
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
