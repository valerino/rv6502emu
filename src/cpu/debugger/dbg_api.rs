/*
 * Filename: /src/cpu/debugger/dbg_api.rs
 * Project: rv6502emu
 * Created Date: 2021-08-28, 06:39:29
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

use crate::cpu::{Cpu, CpuError};

/**
 * assemble opcode at the given address
 */
pub(crate) fn dbg_assemble_opcode(c: &mut Cpu, op: &str, address: u16) -> Result<(), CpuError> {
    // split opcode and operand/s
    let statement = op.trim().to_ascii_lowercase();
    if statement.len() == 0 {
        // done
         res = false;
        break 'assembler;
    }
                let (mut opcode, tmp) = statement.split_once(' ').unwrap_or_default();
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
                    opcode = &statement;
                } else if operand_s.starts_with("($") && operand_s.ends_with(",x)") {
                    // absolute indirect x (65c02)
                    mode_id = AddressingModeId::Aix;
                    operand_s.truncate(operand_s.len() - 3);
                    operand_s.remove(0);
                    operand_s.remove(0);
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
                } else if operand_s.starts_with("$(") && operand_s.len() <= 5 {
                    // indirect ZP (65c02)
                    mode_id = AddressingModeId::Izp;
                    operand_s.truncate(operand_s.len() - 1);
                    operand_s.remove(0);
                    operand_s.remove(0);
                } else if operand_s.contains(",$") {
                    // zeropage relative (65c02)
                    mode_id = AddressingModeId::Zpr;
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
                    println!("invalid opcode!");
                    continue 'assembler;
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
                        println!("{}", e);
                        continue 'assembler;
                    }
                    Ok(()) => (),
                };
    
                // find a match in the opcode matrix
                let op_byte: u8;
                let _ = match self.find_instruction(&c.cpu_type, &opcode, mode_id) {
                    None => {
                        println!("invalid opcode!");
                        continue 'assembler;
                    }
                    Some((_, idx)) => op_byte = idx,
                };
    
                /*println!(
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
                            res = false;
                            break 'assembler;
                        }
                        addr = addr.wrapping_add(1);
                    }
                    AddressingModeId::Abs
                    | AddressingModeId::Abx
                    | AddressingModeId::Zpr
                    | AddressingModeId::Aix
                    | AddressingModeId::Aby
                    | AddressingModeId::Ind => {
                        if mode_id == AddressingModeId::Zpr {
                            // first split $xx,$yy
                            let v: Vec<&str> = operand_s.split(',').collect();
                            let b1: u8;
                            let b2: u8;
                            // get bytes
                            let _ = match u8::from_str_radix(&v[0][1..], 16) {
                                Err(_) => {
                                    println!("invalid opcode!");
                                    continue 'assembler;
                                }
                                Ok(a) => b1 = a,
                            };
                            let _ = match u8::from_str_radix(&v[1][1..], 16) {
                                Err(_) => {
                                    println!("invalid opcode!");
                                    continue 'assembler;
                                }
                                Ok(a) => b2 = a,
                            };
    
                            // write opcode
                            if c.bus
                                .get_memory()
                                .write_byte(addr as usize, op_byte)
                                .is_err()
                            {
                                res = false;
                                break 'assembler;
                            }
                            addr = addr.wrapping_add(1);
    
                            // write zeropage address
                            if c.bus.get_memory().write_byte(addr as usize, b1).is_err() {
                                res = false;
                                break 'assembler;
                            }
                            addr = addr.wrapping_add(1);
    
                            // write offset
                            if c.bus.get_memory().write_byte(addr as usize, b2).is_err() {
                                res = false;
                                break 'assembler;
                            }
                            addr = addr.wrapping_add(1);
                        } else {
                            let _ = match u16::from_str_radix(&operand_s[1..], 16) {
                                Err(_) => {
                                    println!("invalid opcode!");
                                    continue 'assembler;
                                }
                                Ok(a) => {
                                    if c.bus
                                        .get_memory()
                                        .write_byte(addr as usize, op_byte)
                                        .is_err()
                                    {
                                        res = false;
                                        break 'assembler;
                                    }
                                    addr = addr.wrapping_add(1);
                                    if c.bus.get_memory().write_word_le(addr as usize, a).is_err() {
                                        res = false;
                                        break 'assembler;
                                    }
                                    addr = addr.wrapping_add(2);
                                }
                            };
                        }
                    }
                    AddressingModeId::Rel
                    | AddressingModeId::Imm
                    | AddressingModeId::Zpg
                    | AddressingModeId::Zpx
                    | AddressingModeId::Zpy
                    | AddressingModeId::Izp
                    | AddressingModeId::Iny
                    | AddressingModeId::Xin => {
                        let _ = match u8::from_str_radix(&operand_s[1..], 16) {
                            Err(_) => {
                                println!("invalid opcode!");
                                continue 'assembler;
                            }
                            Ok(a) => {
                                if c.bus
                                    .get_memory()
                                    .write_byte(addr as usize, op_byte)
                                    .is_err()
                                {
                                    res = false;
                                    break 'assembler;
                                }
                                addr = addr.wrapping_add(1);
                                if c.bus.get_memory().write_byte(addr as usize, a).is_err() {
                                    res = false;
                                    break 'assembler;
                                }
                                addr = addr.wrapping_add(1);
                            }
                        };
                    }
                };
                if addr < prev_addr {
                    // overlap detected
                    println!("ERROR, overlapping detected!");
                    res = false;
                    break 'assembler;
                }
                prev_addr = addr;
            }
    
    Ok(())
}
