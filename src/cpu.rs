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
use debugger::breakpoints::BreakpointType;
use debugger::Debugger;
pub(crate) mod opcodes;
use std::fmt::{Display, Error, Formatter};

use bitflags::bitflags;
pub(crate) mod addressing_modes;

pub mod cpu_error;
pub mod debugger;
use crate::utils::*;
use cpu_error::{CpuError, CpuErrorType};

/**
 * the cpu registers.
 */
#[derive(Debug, PartialEq)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: CpuFlags,
    pub s: u8,
    pub pc: u16,
}

/**
 * indicates the operation CpuCallbackContext refers to.
 */
#[derive(Debug, PartialEq)]
pub enum CpuOperation {
    Exec,
    Read,
    Write,
    Irq,
    Nmi,
    Brk,
}

/**
 * type of emulated cpu
 */
#[derive(Debug, PartialEq)]
pub enum CpuType {
    /// default, MOS6502
    MOS6502,
    /// WDC 6502C
    WDC65C02,
}

impl Display for CpuType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CpuType::MOS6502 => {
                write!(f, "MOS6502")?;
            }
            CpuType::WDC65C02 => {
                write!(f, "WDC65C02")?;
            }
        };
        Ok(())
    }
}

bitflags! {
    /**
     * flags (values for the P register).
     * https://www.atarimagazines.com/compute/issue53/047_1_All_About_The_Status_Register.php
     */
    pub struct CpuFlags : u8 {
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
pub struct CpuCallbackContext {
    /// address acessed.
    pub address: u16,
    /// access size, may be 1 or 2.
    pub access_size: i8,
    /// first byte (LE) accessed.
    pub value: u8,
    /// one of the CpuOperation enums.
    pub operation: CpuOperation,
}

impl Display for CpuCallbackContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self.operation {
            CpuOperation::Irq | CpuOperation::Nmi => {
                write!(f, "CALLBACK! type={:?}", self.operation)?;
            }
            CpuOperation::Read | CpuOperation::Write => {
                write!(
                    f,
                    "CALLBACK! type={:?}, address=${:04x}, value=${:02x}, access_size={}",
                    self.operation, self.address, self.value, self.access_size
                )?;
            }
            CpuOperation::Brk | CpuOperation::Exec => {
                write!(
                    f,
                    "CALLBACK! type={:?}, address=${:04x}",
                    self.operation, self.address
                )?;
            }
        }
        Ok(())
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "PC: ${:04x}, A: ${:02x}, X: ${:02x}, Y: ${:02x}, S: ${:02x}, P: ${:02x}({})",
            self.pc,
            self.a,
            self.x,
            self.y,
            self.s,
            self.p,
            self.flags_to_string(),
        )?;

        Ok(())
    }
}

impl Registers {
    pub fn new() -> Registers {
        let r = Registers {
            a: 0,
            x: 0,
            y: 0,
            p: CpuFlags::from_bits(0).unwrap(),
            s: 0,
            pc: 0,
        };
        r
    }

    /**
     * convert P (flags) register to a meaningful string
     */
    fn flags_to_string(&self) -> String {
        let s = format!(
            "{}{}{}{}{}{}{}{}",
            if self.p.contains(CpuFlags::N) {
                "N"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::V) {
                "V"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::U) {
                "U"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::B) {
                "B"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::D) {
                "D"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::I) {
                "I"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::Z) {
                "Z"
            } else {
                "-"
            },
            if self.p.contains(CpuFlags::C) {
                "C"
            } else {
                "-"
            },
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

    /// running under debugger ?
    debug: bool,

    /// forces run() to exit
    pub done: bool,

    /// the bus.
    pub bus: Box<dyn Bus>,

    /// callback for the user (optional).
    cb: Option<fn(c: &mut Cpu, cb: CpuCallbackContext)>,
    /// set if irq() must be called within the run loop.
    pub must_trigger_irq: bool,
    /// set if nmi() must be called within the run loop.
    pub must_trigger_nmi: bool,
    /// is there an intewrrupt pending ?
    irq_pending: bool,
    /// to handle interrupt return after RTI in certain situations.
    fix_pc_rti: i8,
    /// the emulated cpu type, default MOS6502.
    cpu_type: CpuType,
}

impl Cpu {
    /**
     * activate logging on stdout through env_logger (max level).
     */
    pub fn enable_logging(&self, enable: bool) {
        enable_logging_internal(enable)
    }

    /**
     * call installed cpu callback if any.
     */
    pub(crate) fn call_callback(
        &mut self,
        address: u16,
        value: u8,
        access_size: i8,
        op: CpuOperation,
    ) {
        if self.cb.is_some() {
            // call callback
            let ctx = CpuCallbackContext {
                address: address,
                access_size: access_size,
                value: value,
                operation: op,
            };
            self.cb.unwrap()(self, ctx);
        }
    }

    /**
     * check if cpu flag is set
     */
    pub(crate) fn is_cpu_flag_set(&self, f: CpuFlags) -> bool {
        if self.regs.p.contains(f) {
            return true;
        }
        false
    }

    /**
     * set/unset cpu flag
     */
    pub(crate) fn set_cpu_flags(&mut self, f: CpuFlags, enable: bool) {
        self.regs.p.set(f, enable);
    }

    /**
     * creates a new cpu instance, with the given Bus attached.
     *
     * the provided callback, if any, will be called *after* executing the following:
     *
     * - memory read
     * - memory write
     * - irq
     * - nmi
     * - brk
     */
    pub fn new(
        b: Box<dyn Bus>,
        cb: Option<fn(c: &mut Cpu, cb: CpuCallbackContext)>,
        t: Option<CpuType>,
    ) -> Cpu {
        let c = Cpu {
            regs: Registers::new(),
            cycles: 0,
            bus: b,
            cb: cb,
            done: false,
            debug: false,
            must_trigger_irq: false,
            must_trigger_nmi: false,
            irq_pending: false,
            fix_pc_rti: 0,
            cpu_type: t.unwrap_or(CpuType::MOS6502),
        };
        println!("created new cpu, type={}", c.cpu_type);
        c
    }

    /**
     * creates a new cpu instance (MOS6502), with the given Bus attached, exposing a Memory.
     */
    pub fn new_default(cb: Option<fn(c: &mut Cpu, cb: CpuCallbackContext)>) -> Cpu {
        let m = super::memory::new_default();
        let b = super::bus::new_default(m);
        Cpu::new(b, cb, Some(CpuType::MOS6502))
    }

    /**
     * resets the cpu setting all registers to the initial values.
     *
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
            p: CpuFlags::U | CpuFlags::I,
            s: 0xff,

            // at reset, we read PC from RESET vector
            pc: addr,
        };
        self.cycles = 7;
        self.done = false;
        self.irq_pending = false;
        self.must_trigger_irq = false;
        self.must_trigger_nmi = false;
        self.fix_pc_rti = 0;
        Ok(())
    }

    /**
     * fetch opcode at PC
     */
    pub(crate) fn fetch(&mut self) -> Result<u8, CpuError> {
        let mem = self.bus.get_memory();
        let b = mem.read_byte(self.regs.pc as usize)?;
        Ok(b)
    }

    /**
     * increment pc and the elapsed cycles
     */
    fn inc_pc(&mut self, instr_size: u16, opcode_cycles: usize) {
        // advance pc and increment the elapsed cycles
        self.regs.pc = self.regs.pc.wrapping_add(instr_size);
        self.cycles = self.cycles.wrapping_add(opcode_cycles);
    }

    /**
     * run the cpu for the given cycles, optionally with a debugger attached.
     *
     * pass 0 to run indefinitely.
     *
     * > note that reset() must be called first to set the start address !
     */
    pub fn run(&mut self, debugger: Option<&mut Debugger>, cycles: usize) -> Result<(), CpuError> {
        let mut bp_rw_triggered = false;
        let mut instr_size: i8 = 0;
        // construct an empty, disabled, debugger to use when None is passed in
        let mut empty_dbg = Debugger::new(false);
        let dbg = debugger.unwrap_or(&mut empty_dbg);
        if dbg.enabled {
            self.debug = true;
        }

        let mut silence_output = false;
        let mut is_error = false;
        let mut opcode_cycles: usize = 0;
        let mut run_cycles: usize = 0;
        // loop
        'interpreter: loop {
            // fetch
            let b = self.fetch()?;
            let (opcode_f, in_cycles, add_extra_cycle_on_page_crossing, mrk) =
                if self.cpu_type == CpuType::MOS6502 {
                    opcodes::OPCODE_MATRIX[b as usize]
                } else {
                    opcodes::OPCODE_MATRIX_65C02[b as usize]
                };
            if !is_error {
                if !silence_output && dbg.show_registers_before_opcode {
                    if log_enabled() {
                        // show registers
                        debug_out_registers(self);
                    }
                }

                // check boundaries
                match cpu_error::check_opcode_boundaries(
                    self.bus.get_memory().get_size(),
                    self.regs.pc as usize,
                    mrk.id,
                    CpuErrorType::MemoryRead,
                    None,
                ) {
                    Err(e) => {
                        println!("{}", e);
                        if !self.debug {
                            // unrecoverable
                            break 'interpreter;
                        } else {
                            // either, this will stop in the debugger
                            dbg.going = false;
                            is_error = true;
                            continue 'interpreter;
                        }
                    }
                    Ok(()) => (),
                };

                // decode
                let _ = match opcode_f(
                    self,
                    Some(dbg),
                    b, // the opcode byte
                    0,
                    false,          // extra_cycle_on_page_crossing
                    true,           // decode only
                    silence_output, // quiet
                ) {
                    Err(e) => {
                        println!("{}", e);
                        if !self.debug {
                            // unrecoverable
                            break 'interpreter;
                        } else {
                            // either, this will stop in the debugger
                            dbg.going = false;
                            is_error = true;
                            continue 'interpreter;
                        }
                    }
                    Ok((a, _)) => {
                        instr_size = a;
                    }
                };

                // call callback if any
                self.call_callback(self.regs.pc, 0, 0, CpuOperation::Exec);
                // check if done has been set
                if self.done {
                    // exiting
                    break 'interpreter;
                }

                // check if irq or nmi has to be triggered
                if self.must_trigger_irq || self.must_trigger_nmi {
                    // trigger irq or nmi
                    if self.must_trigger_nmi {
                        self.fix_pc_rti = instr_size;
                        self.nmi(Some(dbg))?;
                        self.must_trigger_nmi = false;
                        if self.must_trigger_irq {
                            // there's an irq pending, CLI opcode will detect it
                            self.irq_pending = true;
                        }
                        self.must_trigger_irq = false;
                        continue 'interpreter;
                    }
                    if self.must_trigger_irq {
                        self.fix_pc_rti = instr_size;
                        self.irq(Some(dbg))?;
                        self.must_trigger_irq = false;
                        self.must_trigger_nmi = false;
                        continue 'interpreter;
                    }
                }

                // check if we have an exec breakpoint at pc
                if self.debug {
                    match dbg.has_enabled_breakpoint(
                        self,
                        self.regs.pc,
                        BreakpointType::EXEC | BreakpointType::NMI | BreakpointType::IRQ,
                    ) {
                        None => (),
                        Some(idx) => {
                            dbg.going = false;
                            if !silence_output {
                                println!("breakpoint {} triggered!", idx);
                            }
                        }
                    };
                }
            } else {
                // we had an error, will break in the debugger below
                is_error = false;
            }

            // handles debugger if any
            let mut cmd = String::from("p");
            if self.debug {
                let mut cmd_res = false;
                while !cmd_res {
                    match dbg.parse_cmd_stdin(self) {
                        Err(_) => {
                            // io error, something's broken really bad .... break
                            break 'interpreter;
                        }
                        Ok((a, b)) => {
                            cmd = a;
                            cmd_res = b;
                        }
                    };
                }
            }
            match cmd.as_ref() {
                "p" => {
                    silence_output = false;
                    if !bp_rw_triggered {
                        // execute decoded instruction
                        let _ = match opcode_f(
                            self,
                            Some(dbg),
                            b, // the opcode byte
                            in_cycles,
                            add_extra_cycle_on_page_crossing,
                            false, // decode only
                            true,  // quiet, do not print instruction again
                        ) {
                            Ok((_instr_size, _out_cycles)) => {
                                instr_size = _instr_size;
                                opcode_cycles = _out_cycles;
                            }
                            Err(e) => {
                                if e.t == CpuErrorType::RwBreakpoint {
                                    // an r/w breakpoint has triggered, opcode has not executed.
                                    if !silence_output {
                                        println!("R/W breakpoint {} triggered!", e.bp_idx);
                                    }
                                    dbg.going = false;
                                    bp_rw_triggered = true;
                                    is_error = true;
                                    continue 'interpreter;
                                } else {
                                    // report error and break
                                    println!("{}", e);
                                    if !self.debug {
                                        // unrecoverable
                                        break;
                                    } else {
                                        // either, this will stop in the debugger
                                        dbg.going = false;
                                        is_error = true;
                                        continue 'interpreter;
                                    }
                                }
                            }
                        };
                    } else {
                        // a bp has triggered, reset conditions, we will then just advance pc and cycles for the decoded instruction
                        bp_rw_triggered = false;
                        is_error = false;
                    }

                    // step, advance pc and increment the elapsed cycles
                    self.inc_pc(instr_size as u16, opcode_cycles);
                    run_cycles = run_cycles.wrapping_add(opcode_cycles);
                    if cycles != 0 && run_cycles >= cycles {
                        // we're done
                        break 'interpreter;
                    }

                    // finally recheck if there was a pending irq re-enabled by CLI
                    if self.must_trigger_irq {
                        self.irq(Some(dbg))?;
                        self.must_trigger_irq = false;
                        self.must_trigger_nmi = false;
                    }
                }
                "q" => {
                    // gracefully exit
                    break 'interpreter;
                }
                "*" => {
                    silence_output = true;
                    bp_rw_triggered = false;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /**
     * internal, triggers irq or nmi
     */
    fn irq_nmi(&mut self, debugger: Option<&mut Debugger>, v: u16) -> Result<(), CpuError> {
        let mut empty_dbg = Debugger::new(false);
        let dbg = debugger.unwrap_or(&mut empty_dbg);
        // push pc and p on stack
        opcodes::push_word_le(self, Some(dbg), self.regs.pc)?;

        // always push P with U(ndefined) set
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        let mut flags = self.regs.p.clone();
        flags.set(CpuFlags::U, true);
        flags.set(CpuFlags::B, false);
        opcodes::push_byte(self, Some(dbg), flags.bits())?;

        // set I
        self.set_cpu_flags(CpuFlags::I, true);

        if self.cpu_type == CpuType::WDC65C02 {
            // clear the D flag
            // http://6502.org/tutorials/65c02opcodes.html
            self.regs.p.set(CpuFlags::D, false);
        }

        // set pc to address contained at vector
        let addr = self.bus.get_memory().read_word_le(v as usize)?;

        // check for deadlock
        if addr == self.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                self.regs.pc,
                None,
            ));
        }
        self.regs.pc = addr;
        Ok(())
    }

    /**
     * triggers an irq.
     */
    pub fn irq(&mut self, debugger: Option<&mut Debugger>) -> Result<(), CpuError> {
        println!("triggering irq !");
        let res = self.irq_nmi(debugger, Vectors::IRQ as u16);
        // call callback if any
        self.call_callback(0, 0, 0, CpuOperation::Irq);
        res
    }

    /**
     * triggers an nmi.
     */
    pub fn nmi(&mut self, debugger: Option<&mut Debugger>) -> Result<(), CpuError> {
        println!("triggering nmi !");
        let res = self.irq_nmi(debugger, Vectors::NMI as u16);

        // call callback if any
        self.call_callback(0, 0, 0, CpuOperation::Nmi);
        res
    }

    /**
     * sets the cpu mode.
     *
     * > this should be called before run()!     
     */
    pub fn set_cpu_type(&mut self, t: CpuType) {
        self.cpu_type = t;
        println!("setting cpu type to {}.", self.cpu_type);
    }
}
