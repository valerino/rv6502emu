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

/*
 * disassemble opcode at the given address, returns a tuple with (instr_size, cycles including extra, name, addressing mode, operand, target address) on success.
pub(crate) fn dbg_disassemble_opcode(
    c: &mut Cpu,
    address: u16,
) -> Result<(i8, usize, String, AddressingModeId, u16, u16), CpuError> {
*/
/**
 * disassemble n instructions at the given address
 */
use std::str::SplitWhitespace;

impl Debugger {
    pub(super) fn cmd_disassemble(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let num_instr_s = it.next().unwrap_or_default();
        let num_instr = i32::from_str_radix(&num_instr_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if num_instr == 0 {
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
        let mut next_addr = addr;
        let mut instr_count: i32 = 0;
        println!(
            "disassembling {} instructions at ${:04x}\n",
            num_instr, next_addr
        );
        loop {
            match dbg_disassemble_opcode(c, next_addr) {
                Err(e) => {
                    res = false;
                    println!("{}", e);
                    break;
                }
                Ok((instr_size, _cycles, name, id, operand, tgt_addr)) => {
                    // build proper string for the addressing mode
                    println!("\t{} ", repr);

                    // next
                    instr_count = instr_count.wrapping_add(1);
                    if instr_count == num_instr {
                        break;
                    }
                    // next instruction
                    let (na, o) = next_addr.overflowing_add(instr_size as u16);
                    if o {
                        // overlap
                        println!("ERROR, overlapping detected!");
                        res = false;
                        break;
                    }
                    next_addr = na;
                }
            };
        }
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
