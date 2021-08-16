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
use crate::cpu::Cpu;
use crate::utils::*;
use bitflags::bitflags;
use std::fmt::Display;
use std::fmt::{Error, Formatter};
use std::io;
use std::io::{BufRead, Write};
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
}

impl Bp {
    /**
     * convert BreakpointType flags to a meaningful string
     */
    fn flags_to_string(&self) -> String {
        let p = BreakpointType::from_bits(self.t).unwrap();
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
        write!(
            f,
            "${:04x} [{},{}]",
            self.address,
            self.flags_to_string(),
            if self.enabled { "enabled" } else { "disabled" }
        )
        .expect("");

        Ok(())
    }
}

impl Debugger {
    /**
     * add a breakpoint
     */
    pub(super) fn cmd_add_breakpoint(
        &mut self,
        c: &mut Cpu,
        cmd: &str,
        mut it: SplitWhitespace<'_>,
    ) {
        // check breakpoint type
        let t: BreakpointType;
        match cmd {
            "bx" => t = BreakpointType::EXEC,
            "br" => t = BreakpointType::READ,
            "bw" => t = BreakpointType::WRITE,
            "brw" | "bwr" => t = BreakpointType::READ | BreakpointType::WRITE,
            _ => {
                self.cmd_invalid();
                return;
            }
        }

        // get address
        let addr_s = it.next().unwrap_or_default();
        let addr: u16;
        if addr_s.len() == 0 || addr_s.chars().next().unwrap_or_default() != '$' {
            self.cmd_invalid();
            return;
        }
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return;
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
                return;
            }
            Ok(_) => (),
        };

        // add breakpoint if not already present
        for (_, bp) in self.breakpoints.iter().enumerate() {
            if bp.address == addr && ((bp.t & t.bits()) != 0) {
                debug_out_text(&"breakpoint already set!");
                return;
            }
        }
        self.breakpoints.push(Bp {
            address: addr,
            t: t.bits(),
            enabled: true,
        });
        debug_out_text(&"breakpoint set!");
    }

    /**
     * check if there's a breakpoint at the given address and it's enabled, and return its index.
     */
    pub(crate) fn has_enabled_breakpoint(&self, addr: u16, t: BreakpointType) -> Option<i8> {
        for (i, bp) in self.breakpoints.iter().enumerate() {
            if bp.address == addr && bp.enabled && ((bp.t & t.bits()) != 0) {
                return Some(i as i8);
            }
        }
        None
    }

    /**
     * list set breakpoints
     */
    pub(super) fn cmd_show_breakpoints(&self) {
        let l = self.breakpoints.len();
        if l == 0 {
            debug_out_text(&"no breakpoints set.");
            return;
        }

        // walk
        debug_out_text(&format!("listing {} breakpoints\n", l));
        for (i, bp) in self.breakpoints.iter().enumerate() {
            debug_out_text(&format!("{}... {}", i, bp));
        }
    }

    /**
     * enable or disable existing breakpoint
     */
    pub(super) fn cmd_enable_disable_delete_breakpoint(
        &mut self,
        mode: &str,
        mut it: SplitWhitespace<'_>,
    ) {
        // get breakpoint number
        let n_s = it.next().unwrap_or_default();
        let n: i8;
        let _ = match i8::from_str_radix(&n_s, 10) {
            Err(_) => {
                self.cmd_invalid();
                return;
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
        }
    }

    /**
     * clear breakpoints list
     */
    pub(super) fn cmd_clear_breakpoints(&mut self) {
        // ask first
        debug_out_text(&"clear breakpoints list ? (y/n)");
        io::stdout().flush().unwrap();
        let mut full_string = String::new();
        let _ = match io::stdin().lock().read_line(&mut full_string) {
            Err(_) => return,
            Ok(_) => (),
        };
        if full_string.trim().eq_ignore_ascii_case("y") {
            self.breakpoints.clear();
            debug_out_text(&"breakpoints list cleared.");
        }
    }
}
