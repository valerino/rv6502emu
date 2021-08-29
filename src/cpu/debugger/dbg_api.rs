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

use crate::cpu::addressing_modes::AddressingModeId;
use crate::cpu::cpu_error::{self, CpuError, CpuErrorType};
use crate::cpu::opcodes;
use crate::cpu::opcodes::OpcodeMarker;
use crate::cpu::{Cpu, CpuType};

/**
 * find instruction in the opcode matrix
 */
fn find_instruction(
    t: &CpuType,
    s: &str,
    id: AddressingModeId,
) -> Option<(&'static OpcodeMarker, u8)> {
    for (i, (_, _, _, mrk)) in if *t == CpuType::MOS6502 {
        opcodes::OPCODE_MATRIX.iter().enumerate()
    } else {
        opcodes::OPCODE_MATRIX_65C02.iter().enumerate()
    } {
        if mrk.name.eq(s) && mrk.id == id {
            return Some((&mrk, i as u8));
        }
    }
    None
}

/**
 * disassemble opcode at the given address, returns the opcode string and the next address on success.
 */
//pub(crate) fn dbg_disassemble_opcode(c: &mut Cpu, op: &str, address: u16) -> Result<u16, CpuError> {

/**
 * assemble opcode statement string at the given address, returns the assembled bytes and the next address on success.
 */
pub(crate) fn dbg_assemble_opcode(
    c: &mut Cpu,
    op: &str,
    address: u16,
) -> Result<(Vec<u8>, u16), CpuError> {
    let mut addr = address;
    let mut ret_vec: Vec<u8> = Vec::new();

    // split opcode and operand/s
    let statement = op.trim().to_ascii_lowercase();
    if statement.len() == 0 {
        return Err(CpuError::new_default(
            CpuErrorType::InvalidOpcode,
            addr,
            None,
        ));
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
    } else if operand_s.starts_with("$") && operand_s.len() == 5 && !operand_s.contains(",") {
        // absolute
        mode_id = AddressingModeId::Abs;
    } else if operand_s.starts_with("$") && operand_s.ends_with(",x") && operand_s.len() > 6 {
        // absolute x
        mode_id = AddressingModeId::Abx;
        operand_s.truncate(operand_s.len() - 2);
    } else if operand_s.starts_with("$") && operand_s.ends_with(",y") && operand_s.len() > 6 {
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
    } else if operand_s.starts_with("$") && operand_s.ends_with(",x") && operand_s.len() <= 5 {
        // zeropage X
        mode_id = AddressingModeId::Zpx;
        operand_s.truncate(operand_s.len() - 2);
    } else if operand_s.starts_with("$") && operand_s.ends_with(",y") && operand_s.len() <= 5 {
        // zeropage Y
        mode_id = AddressingModeId::Zpy;
        operand_s.truncate(operand_s.len() - 2);
    } else {
        //println!("invalid opcode!");
        return Err(CpuError::new_default(
            CpuErrorType::InvalidOpcode,
            addr,
            None,
        ));
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
            return Err(e);
        }
        Ok(()) => (),
    };

    // find a match in the opcode matrix
    let op_byte: u8;
    let _ = match find_instruction(&c.cpu_type, &opcode, mode_id) {
        None => {
            //println!("invalid opcode!");
            return Err(CpuError::new_default(
                CpuErrorType::InvalidOpcode,
                addr,
                None,
            ));
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
            let res = c.bus.get_memory().write_byte(addr as usize, op_byte);
            match res {
                Err(e) => {
                    return Err(e);
                }
                Ok(_) => (),
            };
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
                    Err(e) => {
                        return Err(CpuError::new_default(
                            CpuErrorType::InvalidOpcode,
                            addr,
                            None,
                        ));
                    }
                    Ok(a) => b1 = a,
                };
                let _ = match u8::from_str_radix(&v[1][1..], 16) {
                    Err(_) => {
                        return Err(CpuError::new_default(
                            CpuErrorType::InvalidOpcode,
                            addr,
                            None,
                        ));
                    }
                    Ok(a) => {
                        // opcode
                        ret_vec.push(op_byte);
                        // zeropage address
                        ret_vec.push(b1);
                        // offset
                        ret_vec.push(a);
                    }
                };
            } else {
                // not zpr
                let _ = match u16::from_str_radix(&operand_s[1..], 16) {
                    Err(_) => {
                        return Err(CpuError::new_default(
                            CpuErrorType::InvalidOpcode,
                            addr,
                            None,
                        ));
                    }
                    Ok(a) => {
                        // opcode
                        ret_vec.push(op_byte);
                        // push operand as LE
                        ret_vec.push((a & 0xff) as u8);
                        ret_vec.push((a >> 8) as u8);
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
                    return Err(CpuError::new_default(
                        CpuErrorType::InvalidOpcode,
                        addr,
                        None,
                    ));
                }
                Ok(a) => {
                    // opcode
                    ret_vec.push(op_byte);
                    // push operand
                    ret_vec.push(a);
                }
            };
        }
    }
    Ok((ret_vec, addr))
}
