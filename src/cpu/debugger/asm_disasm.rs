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

use crate::cpu::addressing_modes::AddressingModeId;
use crate::cpu::cpu_error;
use crate::cpu::cpu_error::CpuErrorType;
use crate::cpu::debugger::Debugger;
use crate::cpu::opcodes;
use crate::cpu::opcodes::OpcodeMarker;
use crate::cpu::Cpu;
use crate::utils::*;
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
        let n = u16::from_str_radix(&n_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if n == 0 {
            // invalid command, missing number of instructions to decode
            self.cmd_invalid();
            return false;
        }
        // save current pc
        let prev_pc = c.regs.pc;
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            if addr_s.chars().next().unwrap_or_default() != '$' {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            match u16::from_str_radix(&addr_s[1..], 16) {
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
        c.regs.pc = addr;
        let mut instr_count: u16 = 0;
        debug_out_text(&format!(
            "disassembling (at most) {} instructions at ${:04x}\n",
            n, addr
        ));
        loop {
            // fetch an instruction
            let b: u8;
            match c.fetch() {
                Err(e) => {
                    debug_out_text(&e);
                    return false;
                }
                Ok(ok) => b = ok,
            }
            // get opcode and check access
            let (opcode_f, _, _, mrk) = opcodes::OPCODE_MATRIX[b as usize];
            let instr_size: i8;
            match cpu_error::check_opcode_boundaries(
                c.bus.get_memory().get_size(),
                c.regs.pc as usize,
                mrk.id,
                CpuErrorType::MemoryRead,
                None,
            ) {
                Err(e) => {
                    debug_out_text(&e);
                    break;
                }
                Ok(()) => (),
            };
            // decode
            match opcode_f(c, None, 0, false, true, false) {
                Err(e) => {
                    debug_out_text(&e);
                    break;
                }
                Ok((a, _)) => instr_size = a,
            }

            // next
            instr_count = instr_count.wrapping_add(1);
            if instr_count == n {
                break;
            }

            // next instruction
            let (next_pc, o) = c.regs.pc.overflowing_add(instr_size as u16);
            if o {
                // overlap
                break;
            }
            c.regs.pc = next_pc;
        }

        // restore pc in the end
        c.regs.pc = prev_pc;
        return true;
    }

    /**
     * find instruction in the opcode matrix
     */
    fn find_instruction(&self, s: &str, id: AddressingModeId) -> Option<(&OpcodeMarker, u8)> {
        for (i, (_, _, _, op)) in opcodes::OPCODE_MATRIX.iter().enumerate() {
            if op.name.eq(s) && op.id == id {
                return Some((&op, i as u8));
            }
        }
        None
    }

    /**
     * assemble instruction/s
     *
     * syntax for the assembler is taken from https://www.masswerk.at/6502/6502_instruction_set.html
     *
     * A	    Accumulator	        OPC A           operand is AC (implied single byte instruction)
     * abs	    absolute	        OPC $LLHH	    operand is address $HHLL
     * abs,X	absolute, X-indexed	OPC $LLHH,X	    operand is address; effective address is address incremented by X with carry
     * abs,Y	absolute, Y-indexed	OPC $LLHH,Y	    operand is address; effective address is address incremented by Y with carry
     * #	    immediate	        OPC #$BB	    operand is byte BB
     * impl	    implied	            OPC	            operand implied
     * ind	    indirect	        OPC ($LLHH)	    operand is address; effective address is contents of word at address: C.w($HHLL)
     * X,ind	X-indexed, indirect	OPC ($LL,X)	    operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
     * ind,Y	indirect, Y-indexed	OPC ($LL),Y	    operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
     * rel	    relative	        OPC $BB         branch target is PC + signed offset BB
     * zpg	    zeropage	        OPC $LL	        operand is zeropage address (hi-byte is zero, address = $00LL)
     * zpg,X	zeropage, X-indexed	OPC $LL,X	    operand is zeropage address; effective address is address incremented by X without carry
     * zpg,Y	zeropage, Y-indexed	OPC $LL,Y	    operand is zeropage address; effective address is address incremented by Y without carry
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

        // get the start address
        if addr_s.chars().next().unwrap_or_default() != '$' {
            // invalid command, address invalid
            self.cmd_invalid();
            return false;
        }
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            Ok(a) => addr = a,
        };

        // read from stdin
        debug_out_text(&format!("assembling at ${:04x}, <enter> to stop.", addr));

        // loop
        let mut prev_addr = addr;
        loop {
            // read asm
            print!("?a> ${:04x}: ", addr);
            io::stdout().flush().unwrap();
            let mut full_string = String::new();
            let _ = match io::stdin().lock().read_line(&mut full_string) {
                Err(_) => break,
                Ok(_) => (),
            };
            // split opcode and operand/s
            full_string = full_string.trim().to_ascii_lowercase();
            if full_string.len() == 0 {
                // done
                break;
            }
            let (mut opcode, tmp) = full_string.split_once(' ').unwrap_or_default();
            opcode = &opcode.trim();

            // also ensure there's no whitestpaces in the operands part
            let mut operand_s = tmp.trim().replace(" ", "").replace("\t", "");

            // find addressing mode and instruction length
            let mode_id: AddressingModeId;
            if operand_s.eq("a") {
                // accumulator
                mode_id = AddressingModeId::Acc;
            } else if operand_s.starts_with("$") && operand_s.len() == 5 && !operand_s.contains(",")
            {
                // absolute
                mode_id = AddressingModeId::Abs;
            } else if operand_s.starts_with("$") && operand_s.ends_with(",x") && operand_s.len() > 6
            {
                // absolute x
                mode_id = AddressingModeId::Abx;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("$") && operand_s.ends_with(",y") && operand_s.len() > 6
            {
                // absolute y
                mode_id = AddressingModeId::Aby;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("#$") {
                // immediate
                mode_id = AddressingModeId::Imm;
                operand_s.remove(0);
            } else if opcode.len() == 0 && operand_s.len() == 0 {
                // implied
                mode_id = AddressingModeId::Imp;
                opcode = &full_string;
            } else if operand_s.starts_with("(") && operand_s.ends_with(")") {
                // indirect
                mode_id = AddressingModeId::Ind;
                operand_s.truncate(operand_s.len() - 1);
                operand_s.remove(0);
            } else if operand_s.ends_with(",x)") {
                // X indirect
                mode_id = AddressingModeId::Xin;
                operand_s.truncate(operand_s.len() - 3);
                operand_s.remove(0);
            } else if operand_s.ends_with("),y") {
                // indirect Y
                mode_id = AddressingModeId::Iny;
                operand_s.truncate(operand_s.len() - 3);
                operand_s.remove(0);
            } else if operand_s.starts_with("$") && operand_s.len() <= 3 {
                if opcode.eq("bpl")
                    || opcode.eq("bmi")
                    || opcode.eq("bvc")
                    || opcode.eq("bvs")
                    || opcode.eq("bcc")
                    || opcode.eq("bcs")
                    || opcode.eq("bne")
                    || opcode.eq("beq")
                {
                    // relative
                    mode_id = AddressingModeId::Rel;
                } else {
                    // zeropage
                    mode_id = AddressingModeId::Zpg;
                }
            } else if operand_s.starts_with("$")
                && operand_s.ends_with(",x")
                && operand_s.len() <= 5
            {
                // zeropage X
                mode_id = AddressingModeId::Zpx;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("$")
                && operand_s.ends_with(",y")
                && operand_s.len() <= 5
            {
                // zeropage Y
                mode_id = AddressingModeId::Zpy;
                operand_s.truncate(operand_s.len() - 2);
            } else {
                debug_out_text(&"invalid opcode!");
                continue;
            }

            // check access
            match cpu_error::check_opcode_boundaries(
                c.bus.get_memory().get_size(),
                addr as usize,
                mode_id,
                CpuErrorType::MemoryWrite,
                None,
            ) {
                Err(e) => {
                    debug_out_text(&e);
                    continue;
                }
                Ok(()) => (),
            };

            // find a match in the opcode matrix
            let op_byte: u8;
            let _ = match self.find_instruction(&opcode, mode_id) {
                None => {
                    debug_out_text(&"invalid opcode!");
                    continue;
                }
                Some((_, idx)) => op_byte = idx,
            };

            /*
            println!(
                "opcode: {} (${:02x}) - operand: {} - modeid={:?}",
                opcode, op_byte, operand_s, mode_id
            );*/

            // write
            match mode_id {
                AddressingModeId::Imp | AddressingModeId::Acc => {
                    if c.bus
                        .get_memory()
                        .write_byte(addr as usize, op_byte)
                        .is_err()
                    {
                        break;
                    }
                    addr = addr.wrapping_add(1 as u16);
                }
                AddressingModeId::Abs
                | AddressingModeId::Abx
                | AddressingModeId::Aby
                | AddressingModeId::Ind => {
                    let _ = match u16::from_str_radix(&operand_s[1..], 16) {
                        Err(_) => {
                            debug_out_text(&"invalid opcode!");
                            continue;
                        }
                        Ok(a) => {
                            if c.bus
                                .get_memory()
                                .write_byte(addr as usize, op_byte)
                                .is_err()
                            {
                                break;
                            }
                            addr = addr.wrapping_add(1);
                            if c.bus.get_memory().write_word_le(addr as usize, a).is_err() {
                                break;
                            }
                            addr = addr.wrapping_add(2 as u16);
                        }
                    };
                }
                AddressingModeId::Rel
                | AddressingModeId::Imm
                | AddressingModeId::Zpg
                | AddressingModeId::Zpx
                | AddressingModeId::Zpy
                | AddressingModeId::Iny
                | AddressingModeId::Xin => {
                    let _ = match u8::from_str_radix(&operand_s[1..], 16) {
                        Err(_) => {
                            debug_out_text(&"invalid opcode!");
                            continue;
                        }
                        Ok(a) => {
                            if c.bus
                                .get_memory()
                                .write_byte(addr as usize, op_byte)
                                .is_err()
                            {
                                break;
                            }
                            addr = addr.wrapping_add(1);
                            if c.bus.get_memory().write_byte(addr as usize, a).is_err() {
                                break;
                            }
                            addr = addr.wrapping_add(1 as u16);
                        }
                    };
                }
            };
            if addr < prev_addr {
                // overlap detected
                break;
            }
            prev_addr = addr;
        }
        return true;
    }
}
