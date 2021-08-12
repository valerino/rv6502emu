/*
 * Filename: /src/debug.rs
 * Project: rv6502emu
 * Created Date: 2021-08-10, 08:46:47
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

use crate::cpu::addressing_modes::AddressingMode;
use crate::cpu::cpu_error::CpuError;
use crate::cpu::opcodes;
use crate::cpu::Cpu;
use crate::memory::Memory;
use hexplay::HexViewBuilder;
use log::*;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead, Write};
use std::str::SplitWhitespace;
use std::u16;

impl Cpu {
    /**
     * enable debugger
     */
    pub fn enable_debugger(&mut self, enable: bool) {
        self.debug = enable
    }

    /**
     * activate logging on stdout trough env_logger (max level)
     */
    pub fn enable_logging(&self, enable: bool) {
        if enable == true {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::max())
                .try_init();
            log::set_max_level(log::LevelFilter::max());
        } else {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::Off)
                .try_init();
            log::set_max_level(log::LevelFilter::Off);
        }
    }

    /**
     * display opcode string, currently implemented to stdout
     */
    pub(crate) fn debug_out_opcode<A: AddressingMode>(
        &mut self,
        opcode_name: &str,
    ) -> Result<(), CpuError> {
        if log::log_enabled!(Level::max()) || self.debug {
            let opc_string = A::repr(self, opcode_name)?;
            //debug!("\t{}", opc_string);
            println!("\t{}", opc_string);
        }
        Ok(())
    }

    /**
     * display opcode string, currently implemented to stdout
     */
    pub(crate) fn debug_out_text(&self, d: &dyn Display) {
        if log::log_enabled!(Level::max()) || self.debug {
            //debug!("{}", s);
            println!("{}", d);
        }
    }

    /**
     * display registers, currently implemented to stdout
     */
    pub(crate) fn debug_out_registers(&self) {
        if log::log_enabled!(Level::max()) || self.debug {
            //debug!("{}", self.regs);
            println!("\t{}", self.regs);
        }
    }

    /**
     * print help banner
     */
    fn dbg_cmd_show_help(&self) {
        self.debug_out_text(&"debugger supported commands: ");
        self.debug_out_text(
            &"\td <# instr> [$address] .... disassemble <# instructions> bytes at <$address>.",
        );
        self.debug_out_text(&"\th ......................... this help.");
        self.debug_out_text(&"\tr ......................... show registers.");
        self.debug_out_text(&"\tp ......................... step (execute next instruction).");
        self.debug_out_text(&"\tt [$address] .............. reset (restart from given [$address], or defaults to reset vector).");
        self.debug_out_text(&"\tq ......................... exit emulator.");
        self.debug_out_text(&"\tv <a|x|y|s|p|pc> <$value>.. set register value.");
        self.debug_out_text(&"\tx <len> <$address> ........ dump <len> bytes at <$address>.");
    }

    /**
     * perform cpu reset
     */
    fn dbg_cmd_reset(&mut self, mut it: SplitWhitespace<'_>) {
        let s = it.next().unwrap_or_default();
        if s.len() > 0 {
            if s.chars().next().unwrap_or_default() != '$' {
                // invalid
                self.dbg_cmd_invalid();
                return;
            }
            // use provided address
            let addr = u16::from_str_radix(&s[1..], 16).unwrap_or_default();
            self.debug_out_text(&format!("cpu reset, restarting at PC=${:04x}.", addr));
            self.reset(Some(addr)).unwrap_or(());
            return;
        }

        // use the reset vector as default
        self.debug_out_text(&"cpu reset, restarting at RESET vector.");
        self.reset(None).unwrap_or(());
    }

    /**
     * disassemble n instructions at the given address
     */
    fn dbg_disassemble(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let n_s = it.next().unwrap_or_default();
        let n = u16::from_str_radix(&n_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if n == 0 {
            // invalid command, missing number of instructions to decode
            self.dbg_cmd_invalid();
            return;
        }
        // save current pc
        let prev_pc = self.regs.pc;
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            if addr_s.chars().next().unwrap_or_default() != '$' {
                // invalid command, addressi invalid
                self.dbg_cmd_invalid();
                return;
            }
            addr = u16::from_str_radix(&addr_s[1..], 16).unwrap_or_default();
        } else {
            // defaults to pc
            addr = self.regs.pc;
        }

        // disassemble
        self.regs.pc = addr;
        let mut instr_count = 0;
        self.debug_out_text(&format!(
            "disassembling {} instructions at ${:04x} (may overlap)\n",
            n, addr
        ));
        loop {
            // fetch an instruction
            let b = self.fetch().unwrap_or_else(|e| {
                self.debug_out_text(&e);
                0
            });

            // decode
            let (opcode_f, _, _) = opcodes::OPCODE_MATRIX[b as usize];
            let (instr_size, _) = opcode_f(self, 0, false, true).unwrap();

            instr_count += 1;
            if instr_count == n {
                break;
            }

            // next instruction
            self.regs.pc = self.regs.pc.wrapping_add(instr_size as u16);
        }

        // restore pc in the end
        self.regs.pc = prev_pc;
    }

    /**
     * hexdump n bytes at the given address
     */
    fn dbg_dump(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let len_s = it.next().unwrap();
        let mut num_bytes = u16::from_str_radix(&len_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if num_bytes == 0 {
            // invalid command, missing number of bytes to dump
            self.dbg_cmd_invalid();
            return;
        }
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            if addr_s.chars().next().unwrap_or_default() != '$' {
                // invalid command, address invalid
                self.dbg_cmd_invalid();
                return;
            }
            let _ = match u16::from_str_radix(&addr_s[1..], 16) {
                Err(_) => {
                    // invalid command, address invalid
                    self.dbg_cmd_invalid();
                    return;
                }
                Ok(a) => addr = a,
            };
        } else {
            // defaults to pc
            addr = self.regs.pc;
        }

        // get the end address
        let mut addr_end: u16 = addr.wrapping_add(num_bytes as u16).wrapping_sub(1);
        let mem = self.bus.get_memory();
        let mut overlap = false;
        if addr_end < addr {
            // address wrapped, use max memory size as the end
            num_bytes = (mem.get_size() as u16).wrapping_sub(addr);
            addr_end = addr.wrapping_add(num_bytes).wrapping_sub(1);
            overlap = true;
        }
        println!("{:04x} {:04x}", addr, addr_end);
        let m_slice = &mem.as_vec()[addr as usize..=addr_end as usize];

        // dump!
        // MAYBE_FIX: if not using a copy, borrow checker complains of mutable reference to *self used twice (due to self.bus.get_memory())
        let mut sl = vec![0; m_slice.len()];
        sl.copy_from_slice(&m_slice);
        self.debug_out_text(&format!(
            "dumping {} bytes at ${:04x}, overlap={}\n",
            num_bytes, addr, overlap
        ));
        let dump = HexViewBuilder::new(&sl)
            .address_offset(addr as usize)
            .row_width(16)
            .finish();

        self.debug_out_text(&dump);
    }

    /**
     * edit cpu registers
     */
    fn dbg_edit_regs(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let reg = it.next().unwrap_or_default();
        let val = it.next().unwrap_or_default();
        if reg.len() == 0 || val.len() == 0 || val.chars().next().unwrap_or_default() != '$' {
            // invalid command, missing value
            self.dbg_cmd_invalid();
            return;
        }

        // match registers and assign value
        let c = reg.chars().next().unwrap_or_default();
        match c {
            'a' => {
                let _ = match u8::from_str_radix(&val[1..], 16) {
                    Err(_) => {
                        // invalid command, value invalid
                        self.dbg_cmd_invalid();
                        return;
                    }
                    Ok(a) => {
                        self.debug_out_text(&format!("setting register 'a' to ${:02x}", a));
                        self.regs.a = a;
                    }
                };
            }
            'x' => {
                let _ = match u8::from_str_radix(&val[1..], 16) {
                    Err(_) => {
                        // invalid command, value invalid
                        self.dbg_cmd_invalid();
                        return;
                    }
                    Ok(x) => {
                        self.debug_out_text(&format!("setting register 'x' to ${:02x}", x));
                        self.regs.x = x;
                    }
                };
            }
            'y' => {
                let _ = match u8::from_str_radix(&val[1..], 16) {
                    Err(_) => {
                        // invalid command, value invalid
                        self.dbg_cmd_invalid();
                        return;
                    }
                    Ok(y) => {
                        self.debug_out_text(&format!("setting register 'y' to ${:02x}", y));
                        self.regs.y = y;
                    }
                };
            }
            's' => {
                let _ = match u8::from_str_radix(&val[1..], 16) {
                    Err(_) => {
                        // invalid command, value invalid
                        self.dbg_cmd_invalid();
                        return;
                    }
                    Ok(s) => {
                        self.debug_out_text(&format!("setting register 's' to ${:02x}", s));
                        self.regs.s = s;
                    }
                };
            }
            'p' => {
                if Some(reg.chars().nth(1).unwrap_or_default()).unwrap_or_default() == 'c' {
                    let _ = match u16::from_str_radix(&val[1..], 16) {
                        Err(_) => {
                            // invalid command, value invalid
                            self.dbg_cmd_invalid();
                            return;
                        }
                        Ok(pc) => {
                            self.debug_out_text(&format!("setting register 'pc' to ${:02x}", pc));
                            self.regs.pc = pc;
                        }
                    };
                } else {
                    let _ = match u8::from_str_radix(&val[1..], 16) {
                        Err(_) => {
                            // invalid command, value invalid
                            self.dbg_cmd_invalid();
                            return;
                        }
                        Ok(p) => {
                            self.debug_out_text(&format!("setting register 'p' to ${:02x}", p));
                            self.regs.p = p;
                        }
                    };
                }
            }
            _ => self.dbg_cmd_invalid(),
        }
    }

    /**
     * report invalid command
     */
    fn dbg_cmd_invalid(&self) {
        self.debug_out_text(&"invalid command, try 'h' for help !");
    }

    /**
     * handle debugger input from stdin, if debugger is active.
     *
     * returns the debugger command ('q' on exit, '*' for no-op)
     */
    pub(crate) fn handle_debugger_input_stdin(&mut self) -> Result<char, std::io::Error> {
        if self.debug {
            // read from stdin
            let mut full_string = String::new();
            print!("?:> ");
            io::stdout().flush().unwrap();
            io::stdin().lock().read_line(&mut full_string)?;

            // split command and parameters
            let mut it = full_string.split_whitespace();
            let cmd = it.next().unwrap_or_default().to_ascii_lowercase();

            // handle command
            match cmd.trim() {
                // help
                "d" => {
                    self.dbg_disassemble(it);
                    return Ok('*');
                }
                // help
                "h" => {
                    self.dbg_cmd_show_help();
                    return Ok('*');
                }
                // quit
                "q" => {
                    self.debug_out_text(&"quit!");
                    return Ok('q');
                }
                // show registers
                "r" => {
                    self.debug_out_registers();
                    return Ok('*');
                }
                // step
                "p" => return Ok('p'),
                // reset
                "t" => {
                    self.dbg_cmd_reset(it);
                    return Ok('*');
                }
                // edit registers
                "v" => {
                    self.dbg_edit_regs(it);
                    return Ok('*');
                }
                "x" => {
                    self.dbg_dump(it);
                    return Ok('*');
                }
                // invalid
                _ => {
                    self.dbg_cmd_invalid();
                    return Ok('*');
                }
            }
        }

        // default returns 'no-op'
        Ok('*')
    }
}
