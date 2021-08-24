/*
 * Filename: /src/debugger/breakpoints.rs
 * Project: rv6502emu
 * Created Date: 2021-08-16, 09:25:46
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

use crate::cpu::cpu_error;
use crate::cpu::cpu_error::CpuErrorType;
use crate::cpu::debugger::Debugger;
use crate::cpu::CpuError;
use crate::cpu::{Cpu, Registers, Vectors};
use crate::utils::*;
use bitflags::bitflags;
use std::fmt::Display;
use std::fmt::{Error, Formatter};
use std::io;
use std::io::{BufRead, Write};
use std::str::Split;
use std::str::SplitWhitespace;

bitflags! {
    /**
     * flags for breakpoint types
     */
    pub(crate) struct BreakpointType : u8 {
        /// triggers on execute.
        const EXEC = 0b00000001;

        /// triggers on memory read.
        const READ = 0b00000010;

        /// triggers on memory write.
        const WRITE = 0b00000100;

        /// triggers on irq.
        const IRQ =   0b00001000;

        /// triggers on nmi.
        const NMI =   0b00010000;
    }
}

/**
 * represents a breakpoint
 */
#[derive(PartialEq, Debug)]
pub(crate) struct Bp {
    pub(super) address: u16,
    pub(super) t: u8,
    pub(super) enabled: bool,
    pub(super) regs: Option<Registers>,
    pub(super) cycles: usize,
}

impl Bp {
    /**
     * convert BreakpointType flags to a meaningful string
     */
    fn flags_to_string(&self) -> String {
        let p = BreakpointType::from_bits(self.t).unwrap();
        // nmi and irq are single
        if p.contains(BreakpointType::NMI) {
            return String::from("NMI");
        }
        if p.contains(BreakpointType::IRQ) {
            return String::from("IRQ");
        }

        let s = format!(
            "{}{}{}",
            if p.contains(BreakpointType::READ) {
                "R"
            } else {
                "-"
            },
            if p.contains(BreakpointType::WRITE) {
                "W"
            } else {
                "-"
            },
            if p.contains(BreakpointType::EXEC) {
                "X"
            } else {
                "-"
            },
        );
        s
    }
}

impl Display for Bp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.t == BreakpointType::NMI.bits() || self.t == BreakpointType::IRQ.bits() {
            if self.regs.is_some() {
                write!(
                    f,
                    "[{},{}], conditions: {}{}",
                    self.flags_to_string(),
                    if self.enabled { "enabled" } else { "disabled" },
                    self.regs.as_ref().unwrap(),
                    if self.cycles != 0 {
                        format!(", cycles={}", self.cycles)
                    } else {
                        "".to_string()
                    },
                )
                .expect("");
            } else {
                write!(
                    f,
                    "[{},{}]",
                    self.flags_to_string(),
                    if self.enabled { "enabled" } else { "disabled" }
                )
                .expect("");
            }
        } else {
            if self.regs.is_some() {
                write!(
                    f,
                    "${:04x} [{},{}], conditions: {}{}",
                    self.address,
                    self.flags_to_string(),
                    if self.enabled { "enabled" } else { "disabled" },
                    self.regs.as_ref().unwrap(),
                    if self.cycles != 0 {
                        format!(", cycles={}", self.cycles)
                    } else {
                        "".to_string()
                    },
                )
                .expect("");
            } else {
                write!(
                    f,
                    "${:04x} [{},{}]",
                    self.address,
                    self.flags_to_string(),
                    if self.enabled { "enabled" } else { "disabled" }
                )
                .expect("");
            }
        }

        Ok(())
    }
}

impl Debugger {
    /**
     * check if an rw breakpoint triggers at address (returns a CpuError::RwBreakpoint in case)
     */
    pub(crate) fn handle_rw_breakpoint(
        &self,
        c: &Cpu,
        address: u16,
        t: BreakpointType,
    ) -> Result<(), CpuError> {
        // check if a breakpoint has to be triggered
        match self.has_enabled_breakpoint(c, address, t) {
            Some(idx) => {
                // trigger!
                let e = CpuError {
                    t: CpuErrorType::RwBreakpoint,
                    address: address as usize,
                    mem_size: 0,
                    access_size: 1,
                    bp_idx: idx,
                    msg: None,
                };
                return Err(e);
            }
            None => (),
        };

        Ok(())
    }

    /**
     * split a string like "a=$10,x=$20,cycles=1234,..." and build a breakpoint with conditions
     */
    fn bp_from_conditions<'a>(&mut self, itt: &mut Split<'a, char>, bp: &mut Bp) -> bool {
        let mut count = 0;
        let mut target_regs = Registers {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: 0,
            pc: 0,
        };
        let mut target_cycles: usize = 0;
        loop {
            // get entry
            let item = itt.next().unwrap_or_default().to_ascii_lowercase();
            if item.len() == 0 {
                break;
            }

            // split with "="
            let arr: Vec<&str> = item.split('=').collect();
            if arr.len() != 2 {
                // wrong condition
                return false;
            }
            match arr[0] {
                "a" => {
                    target_regs.a = u8::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "x" => {
                    target_regs.x = u8::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "y" => {
                    target_regs.y = u8::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "s" => {
                    target_regs.s = u8::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "p" => {
                    target_regs.p = u8::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "pc" => {
                    target_regs.pc = u16::from_str_radix(&arr[1][is_dollar_hex(&arr[1])..], 16)
                        .unwrap_or_default();
                }
                "cycles" => {
                    target_cycles = usize::from_str_radix(&arr[1], 10).unwrap_or_default();
                }
                _ => {
                    // invalid
                    return false;
                }
            }

            // next item
            count += 1;
        }
        if count == 0 {
            // invalid, no items
            return false;
        }

        // return the filled bp struct
        bp.regs = Some(target_regs);
        bp.cycles = target_cycles;
        return true;
    }

    /**
     * add a breakpoint.
     *
     * > exec, nmi, irq breakpoints triggers BEFORE the instruction executes. read/write breakpoints triggers AFTER the instruction executed.
     */
    pub(super) fn cmd_add_breakpoint(
        &mut self,
        c: &mut Cpu,
        cmd: &str,
        mut it: SplitWhitespace<'_>,
    ) -> bool {
        // check breakpoint type
        let t: BreakpointType;
        match cmd {
            "bx" => t = BreakpointType::EXEC,
            "bn" => t = BreakpointType::NMI,
            "bq" => t = BreakpointType::IRQ,
            "br" => t = BreakpointType::READ,
            "bw" => t = BreakpointType::WRITE,
            "brw" => t = BreakpointType::READ | BreakpointType::WRITE,
            _ => {
                self.cmd_invalid();
                return false;
            }
        }

        // check if type is irq or nmi, so compute the address
        let addr: u16;
        if t == BreakpointType::IRQ {
            match c.bus.get_memory().read_word_le(Vectors::IRQ as usize) {
                Ok(a) => addr = a,
                Err(_) => {
                    self.cmd_invalid();
                    return false;
                }
            };
        } else if t == BreakpointType::NMI {
            match c.bus.get_memory().read_word_le(Vectors::NMI as usize) {
                Ok(a) => addr = a,
                Err(_) => {
                    self.cmd_invalid();
                    return false;
                }
            };
        } else {
            // get address from iterator
            let addr_s = it.next().unwrap_or_default();
            if addr_s.len() == 0 {
                self.cmd_invalid();
                return false;
            }
            let _ = match u16::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
                Err(_) => {
                    // invalid command, address invalid
                    self.cmd_invalid();
                    return false;
                }
                Ok(a) => addr = a,
            };
            let _ = match cpu_error::check_address_boundaries(
                c.bus.get_memory().get_size(),
                addr as usize,
                1,
                CpuErrorType::MemoryRead,
                None,
            ) {
                Err(e) => {
                    debug_out_text(&e);
                    return false;
                }
                Ok(_) => (),
            };
        }

        // add breakpoint if not already present
        for (_, bp) in self.breakpoints.iter().enumerate() {
            if bp.address == addr && ((bp.t & t.bits()) != 0) {
                debug_out_text(&"breakpoint already set!");
                return false;
            }
        }

        let mut bp = Bp {
            address: addr,
            t: t.bits(),
            enabled: true,
            regs: None,
            cycles: 0,
        };

        // check if we have conditions
        let conditions = it.next().unwrap_or_default();
        if !conditions.is_empty() {
            // split commas and build proper bp struct
            let mut itt = conditions.split(',');
            if !self.bp_from_conditions(&mut itt, &mut bp) {
                // invalid command
                self.cmd_invalid();
                return false;
            }
        }

        debug_out_text(&format!("breakpoint set! ({})", bp));
        self.breakpoints.push(bp);
        return true;
    }

    /**
     * check if there's a breakpoint at the given address and it's enabled, and return its index.
     */
    pub(crate) fn has_enabled_breakpoint(
        &self,
        c: &Cpu,
        addr: u16,
        t: BreakpointType,
    ) -> Option<i8> {
        for (i, bp) in self.breakpoints.iter().enumerate() {
            if (bp.address == addr || bp.cycles != 0 && bp.cycles == c.cycles)
                && bp.enabled
                && ((bp.t & t.bits()) != 0)
            {
                // check conditions too
                if bp.regs.is_some() {
                    let checks = bp.regs.as_ref().unwrap();
                    if checks.a == c.regs.a
                        || checks.x == c.regs.x
                        || checks.y == c.regs.y
                        || checks.s == c.regs.s
                        || checks.p == c.regs.p
                        || checks.a == c.regs.a
                        || checks.pc == c.regs.pc
                    {
                        // triggered with registers conditions
                        return Some(i as i8);
                    }
                } else {
                    return Some(i as i8);
                }
            }
        }
        None
    }

    /**
     * list set breakpoints
     */
    pub(super) fn cmd_show_breakpoints(&self) -> bool {
        let l = self.breakpoints.len();
        if l == 0 {
            debug_out_text(&"no breakpoints set.");
            return false;
        }

        // walk
        debug_out_text(&format!("listing {} breakpoints\n", l));
        for (i, bp) in self.breakpoints.iter().enumerate() {
            debug_out_text(&format!("{}... {}", i, bp));
        }
        return true;
    }

    /**
     * enable or disable existing breakpoint
     */
    pub(super) fn cmd_enable_disable_delete_breakpoint(
        &mut self,
        mode: &str,
        mut it: SplitWhitespace<'_>,
    ) -> bool {
        // get breakpoint number
        let n_s = it.next().unwrap_or_default();
        let n: i8;
        let _ = match i8::from_str_radix(&n_s, 10) {
            Err(_) => {
                self.cmd_invalid();
                return false;
            }
            Ok(a) => n = a,
        };

        let action: &str;
        if self.breakpoints.len() >= (n as usize + 1) {
            if mode.eq("be") {
                // enable
                self.breakpoints[n as usize].enabled = true;
                action = "enabled";
            } else if mode.eq("bd") {
                // disable
                self.breakpoints[n as usize].enabled = false;
                action = "disabled";
            } else {
                // delete
                self.breakpoints.remove(n as usize);
                action = "deleted";
            }
            debug_out_text(&format!("breakpoint {} has been {}.", n, action));
        } else {
            // invalid size
            self.cmd_invalid();
            return false;
        }
        return true;
    }

    /**
     * clear breakpoints list
     */
    pub(super) fn cmd_clear_breakpoints(&mut self) -> bool {
        // ask first
        debug_out_text(&"delete all breakpoints ? (y/n)");
        io::stdout().flush().unwrap();
        let mut full_string = String::new();
        let _ = match io::stdin().lock().read_line(&mut full_string) {
            Err(_) => return false,
            Ok(_) => (),
        };
        if full_string.trim().eq_ignore_ascii_case("y") {
            self.breakpoints.clear();
            debug_out_text(&"breakpoints cleared.");
            return true;
        }
        return false;
    }
}
