/*
 * Filename: /src/debugger/asm_disasm.rs
 * Project: rv6502emu
 * Created Date: 2021-08-16, 11:14:58
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

use crate::cpu::debugger::Debugger;
use crate::cpu::Cpu;
use crate::utils::*;
#[path = "./dbg_api.rs"]
mod dbg_api;
use dbg_api::*;
use std::io;
use std::io::{BufRead, Write};

/**
 * disassemble n instructions at the given address
 */
use std::str::SplitWhitespace;

impl Debugger {
    pub(super) fn cmd_disassemble(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let n_s = it.next().unwrap_or_default();
        let n = i32::from_str_radix(&n_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if n == 0 {
            // invalid command, missing number of instructions to decode
            self.cmd_invalid();
            return false;
        }
        let mut res = true;
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            match u16::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
                Err(_) => {
                    // invalid command, address invalid
                    self.cmd_invalid();
                    return false;
                }
                Ok(a) => addr = a,
            }
        } else {
            // defaults to pc
            addr = c.regs.pc;
        }

        // disassemble
        let prev_pc = c.regs.pc;
        c.regs.pc = addr;
        let mut instr_count: i32 = 0;
        println!("disassembling {} instructions at ${:04x}\n", n, addr);
        loop {
            match dbg_disassemble_opcode(c, c.regs.pc) {
                Err(e) => {
                    res = false;
                    println!("{}", e);
                    break;
                }
                Ok((instr_size, _cycles, repr)) => {
                    println!("\t{}", repr);

                    // next
                    instr_count = instr_count.wrapping_add(1);
                    if instr_count == n {
                        break;
                    }
                    // next instruction
                    let (next_pc, o) = c.regs.pc.overflowing_add(instr_size as u16);
                    if o {
                        // overlap
                        println!("ERROR, overlapping detected!");
                        res = false;
                        break;
                    }
                    c.regs.pc = next_pc;
                }
            };
        }

        // restore pc
        c.regs.pc = prev_pc;
        return res;
    }

    /**
     * assemble instruction/s
     */
    pub(super) fn cmd_assemble(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let addr_s = it.next().unwrap_or_default();
        let mut addr: u16;
        if addr_s.len() == 0 {
            // invalid command, address invalid
            self.cmd_invalid();
            return false;
        }

        match u16::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            Ok(a) => addr = a,
        };

        // read from stdin
        println!("assembling at ${:04x}, <enter> to stop.", addr);

        // loop
        let mut prev_addr = addr;
        let mut assemble_res = true;
        loop {
            // read from stdin
            print!("?a> ${:04x}: ", addr);
            io::stdout().flush().unwrap();
            let mut statement = String::new();
            let _ = match io::stdin().lock().read_line(&mut statement) {
                Err(_) => {
                    assemble_res = false;
                    break;
                }
                Ok(_) => {
                    if statement.trim().len() == 0 {
                        break;
                    }
                }
            };
            match dbg_api::dbg_assemble_opcode(c, statement.as_ref(), addr) {
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
                Ok(v) => {
                    // write memory and continue from the next address
                    for (i, b) in v.iter().enumerate() {
                        match c
                            .bus
                            .get_memory()
                            .write_byte(addr.wrapping_add(i as u16) as usize, *b)
                        {
                            Err(e) => {
                                println!("{}", e);
                                assemble_res = false;
                                break;
                            }
                            Ok(_) => (),
                        };
                    }

                    // next
                    addr = addr.wrapping_add(v.len() as u16);
                    if addr < prev_addr {
                        // overlap detected
                        println!("ERROR, overlapping detected!");
                        assemble_res = false;
                        break;
                    }
                    prev_addr = addr;
                }
            };
        }
        assemble_res
    }
}
