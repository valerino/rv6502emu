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
use crate::cpu::addressing_modes::*;
use crate::cpu::cpu_error::{self, CpuError, CpuErrorType};
use crate::cpu::opcodes;
use crate::cpu::{Cpu, CpuType};

/**
 * get opcode information from opcode byte at addr.
 *
 *
 * returns a tuple with (instr_size, cycles including extra, opcode name, operand string, operand, addressing mode, target address)
 */
fn get_opcode_info(
    c: &mut Cpu,
    addr: u16,
) -> Result<(i8, usize, String, String, u16, AddressingModeId, u16), CpuError> {
    let b = c.bus.get_memory().read_byte(addr as usize)?;
    // fetch the opcode
    let (_, cycles, has_extra_cycle_on_page_crossing, name, id) = if c.cpu_type == CpuType::MOS6502
    {
        OPCODE_MATRIX[opcode_byte as usize]
    } else {
        OPCODE_MATRIX_65C02[opcode_byte as usize]
    };

    let (tgt_addr, cyc) = A::target_address(c, cycles, has_extra_cycle_on_page_crossing)?;
    let op_string: String;
    let tgt_addr: u16;
    let cycles_total: u16;
    let instr_len: i8;
    let op: u16;
    match id {
        AddressingModeId::Acc => {
            op_string = String::from("");
            let op = A::operand(c)?;
        }
        AddressingModeId::Abs => {
            op_string = format!("${:04x}", op);
        }
        AddressingModeId::Abx => {
            op_string = format!("${:04x},X", op);
        }
        AddressingModeId::Aby => {
            op_string = format!("${:04x},Y", op);
        }
        AddressingModeId::Aix => {
            op_string = format!("(${:04x},X)", op);
        }
        AddressingModeId::Imm => {
            op_string = format!("#${:02x}", op);
        }
        AddressingModeId::Imp => {
            op_string = String::from("");
        }
        AddressingModeId::Ind => {
            op_string = format!("(${:04x})", op);
        }
        AddressingModeId::Izp => {
            op_string = format!("(${:02x})", op);
        }
        AddressingModeId::Xin => {
            op_string = format!("(${:02x},X)", op);
        }
        AddressingModeId::Iny => {
            op_string = format!("(${:02x}),Y", op);
        }
        AddressingModeId::Rel => {
            op_string = format!("${:04x}", op);
        }
        AddressingModeId::Zpg => {
            op_string = format!("${:02x}", op);
        }
        AddressingModeId::Zpx => {
            op_string = format!("${:02x},X", op);
        }
        AddressingModeId::Zpy => {
            op_string = format!("${:02x},Y", op);
        }
        AddressingModeId::Zpr => {
            // pc+1=byte to test
            // pc+2=pc-relative offset to branch to
            let b1: u16 = op >> 8;
            let b2: u16 = op & 0xff;
            op_string = format!("${:02x},${:04x}", b1, b2);
        }
    }

    Ok((
        A::len(),
        cyc,
        String::from(name).to_uppercase(),
        op_string,
        op,
        id,
        tgt_addr,
    ))
}

/**
 * disassemble opcode at the given address,
 * returns a tuple with (instr_size, cycles including extra, opcode name, operand string, operand, addressing mode, target address)
 */
pub(crate) fn dbg_disassemble_opcode(
    c: &mut Cpu,
    address: u16,
) -> Result<(i8, usize, String, String, u16, AddressingModeId, u16), CpuError> {
    // fetch the opcode byte and get infos
    let b = c.bus.get_memory().read_byte(address as usize)?;
    opcodes::get_opcode_info(c, b)
}

/**
 * find instruction in the opcode matrix, return the index if found
 */
fn find_instruction(t: CpuType, s: &str, id: AddressingModeId) -> Option<usize> {
    for (i, (_f, _cycles, _extra, name, addr_mode)) in if t == CpuType::MOS6502 {
        opcodes::OPCODE_MATRIX.iter().enumerate()
    } else {
        opcodes::OPCODE_MATRIX_65C02.iter().enumerate()
    } {
        if *addr_mode == id && *name == s {
            return Some(i);
        }
    }
    None
}

/**
 * assemble opcode statement string at the given address, returns a tuple with a Vec with the instruction bytes on success.
 *
 * A	    Accumulator	        OPC A           operand is AC (implied single byte instruction)
 * abs	    absolute	        OPC $addr	    operand is address $HHLL
 * abs,X	absolute, X-indexed	OPC $addr,X	    operand is address; effective address is address incremented by X with carry
 * abs,Y	absolute, Y-indexed	OPC $addr,Y	    operand is address; effective address is address incremented by Y with carry
 * #	    immediate	        OPC #$BB	    operand is byte BB
 * impl	    implied	            OPC	            operand implied
 * ind	    indirect	        OPC ($addr)	    operand is address; effective address is contents of word at address: C.w($HHLL)
 * X,ind	X-indexed, indirect	OPC ($ad,X)	    operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
 * ind,Y	indirect, Y-indexed	OPC ($ad),Y	    operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
 * rel	    relative	        OPC $BB         branch target is PC + signed offset BB
 * zpg	    zeropage	        OPC $LL	        operand is zeropage address (hi-byte is zero, address = $00LL)
 * zpg,X	zeropage, X-indexed	OPC $LL,X	    operand is zeropage address; effective address is address incremented by X without carry
 * zpg,Y	zeropage, Y-indexed	OPC $LL,Y	    operand is zeropage address; effective address is address incremented by Y without carry
 *
 * for 65c02:
 * zpr (ZeroPage relative)      OPC $ad,$BB     operand is zeropage address
 * iax (Indirect Absolute X)    OPC ($addr,X)
 */
pub(crate) fn dbg_assemble_opcode(
    c: &mut Cpu,
    op: &str,
    address: u16,
) -> Result<Vec<u8>, CpuError> {
    let mut ret_vec: Vec<u8> = Vec::new();

    // split opcode and operand/s
    let statement = op.trim().to_ascii_lowercase();
    if statement.len() == 0 {
        return Err(CpuError::new_default(
            CpuErrorType::InvalidOpcode,
            address,
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
            address,
            None,
        ));
    }

    // check access
    match cpu_error::check_opcode_boundaries(
        c.bus.get_memory().get_size(),
        address as usize,
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
    let _ = match find_instruction(c.cpu_type, opcode, mode_id) {
        None => {
            //println!("invalid opcode!");
            return Err(CpuError::new_default(
                CpuErrorType::InvalidOpcode,
                address,
                None,
            ));
        }
        Some(idx) => op_byte = idx as u8,
    };

    /*println!(
        "opcode: {} (${:02x}) - operand: {} - modeid={:?}",
        opcode, op_byte, operand_s, mode_id
    );*/

    // write
    match mode_id {
        AddressingModeId::Imp | AddressingModeId::Acc => {
            // opcode only
            ret_vec.push(op_byte);
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
                // get bytes
                let _ = match u8::from_str_radix(&v[0][1..], 16) {
                    Err(_) => {
                        return Err(CpuError::new_default(
                            CpuErrorType::InvalidOpcode,
                            address,
                            None,
                        ));
                    }
                    Ok(a) => b1 = a,
                };
                let _ = match u8::from_str_radix(&v[1][1..], 16) {
                    Err(_) => {
                        return Err(CpuError::new_default(
                            CpuErrorType::InvalidOpcode,
                            address,
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
                            address,
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
                        address,
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
    Ok(ret_vec)
}
