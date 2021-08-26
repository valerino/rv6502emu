/*
 * Filename: /src/cpu/addressing_modes.rs
 * Project: rv6502emu
 * Created Date: 2021-08-09, 12:52:06
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

use crate::cpu::cpu_error::CpuError;
use crate::cpu::debugger::breakpoints::BreakpointType;
use crate::cpu::debugger::Debugger;
use crate::cpu::{Cpu, CpuOperation, CpuType};
use crate::utils;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

/**
 * this is used by the assembler part to tag elements in the opcode matrix
 */
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum AddressingModeId {
    Acc,
    Abs,
    Abx,
    Aby,
    Imm,
    Imp,
    Ind,
    Xin,
    Iny,
    Rel,
    Zpg,
    Zpx,
    Zpy,
}

impl Display for AddressingModeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            AddressingModeId::Acc => {
                write!(f, "Acc")?;
            }
            AddressingModeId::Abs => {
                write!(f, "Abs")?;
            }
            AddressingModeId::Abx => {
                write!(f, "AbX")?;
            }
            AddressingModeId::Aby => {
                write!(f, "AbY")?;
            }
            AddressingModeId::Imm => {
                write!(f, "Imm")?;
            }
            AddressingModeId::Imp => {
                write!(f, "Imp")?;
            }
            AddressingModeId::Ind => {
                write!(f, "Ind")?;
            }
            AddressingModeId::Xin => {
                write!(f, "Xin")?;
            }
            AddressingModeId::Iny => {
                write!(f, "InY")?;
            }
            AddressingModeId::Rel => {
                write!(f, "Rel")?;
            }
            AddressingModeId::Zpg => {
                write!(f, "Zpg")?;
            }
            AddressingModeId::Zpx => {
                write!(f, "ZpX")?;
            }
            AddressingModeId::Zpy => {
                write!(f, "ZpY")?;
            }
        }
        Ok(())
    }
}

/**
 * http://www.emulator101.com/6502-addressing-modes.html
 * https://www.masswerk.at/6502/6502_instruction_set.html
 */
pub(crate) trait AddressingMode {
    /**
     * the instruction size
     */
    fn len() -> i8 {
        1
    }

    /**
     * string representation
     */
    fn repr(_c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        Ok(String::from(opcode_name.to_uppercase()))
    }

    /**
     * fetch the opcode target address depending on the addressing mode, returns a tuple with (address, extra_cycle_if_page_crossed))
     */
    fn target_address(
        _c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        Ok((0, false))
    }

    /**
     * load byte from address
     */
    fn load(c: &mut Cpu, d: Option<&Debugger>, address: u16) -> Result<u8, CpuError> {
        let m = c.bus.get_memory();

        // read
        let b = m.read_byte(address as usize)?;

        // check if a breakpoint has to be triggered
        if d.is_some() {
            d.unwrap()
                .handle_rw_breakpoint(c, address, BreakpointType::READ)?
        }

        // call callback if any
        c.call_callback(address, b, 1, CpuOperation::Read);
        Ok(b)
    }

    /**
     * store byte to address
     */
    fn store(c: &mut Cpu, d: Option<&Debugger>, address: u16, b: u8) -> Result<(), CpuError> {
        let m = c.bus.get_memory();

        // write
        m.write_byte(address as usize, b)?;

        // check if a breakpoint has to be triggered
        if d.is_some() {
            d.unwrap()
                .handle_rw_breakpoint(c, address, BreakpointType::WRITE)?
        }

        // call callback if any
        c.call_callback(address, b, 1, CpuOperation::Write);
        Ok(())
    }
}

/**
 * check hi-byte of source and destination addresses, to determine if there's a page cross.
 */
fn is_page_cross(src_addr: u16, dst_addr: u16) -> bool {
    if src_addr & 0xff00 == dst_addr & 0xff00 {
        return true;
    }
    false
}

/**
 * get branch target for relative addressing, returns tuple with (new_pc_address, add_extra_cycle)
 */
pub(crate) fn get_relative_branch_target(src_pc: u16, branch_offset: u8) -> (u16, bool) {
    let mut two_compl: u16 = branch_offset as u16;
    if utils::is_signed(branch_offset) {
        // sign extend
        two_compl |= 0xff00;
    }

    // new offset is pc + 2 complement signed offset + sizeof the opcode (which, for relative addressing, is 2)
    let new_pc = src_pc.wrapping_add(two_compl).wrapping_add(2);
    if is_page_cross(src_pc, new_pc) {
        return (new_pc, true);
    }
    (new_pc, false)
}

/**
 * These instructions have register A (the accumulator) as the target. Examples are LSR A and ROL A.
 */
pub(crate) struct AccumulatorAddressing;
impl AddressingMode for AccumulatorAddressing {
    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let b = c.bus.get_memory().read_byte(c.regs.pc as usize)?;
        Ok(format!(
            "${:04x}:\t{:02x}\t\t-->\t{} A\t[{}])",
            c.regs.pc,
            b,
            opcode_name.to_uppercase(),
            AddressingModeId::Acc
        ))
    }

    fn target_address(
        _c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // implied A
        Ok((0, false))
    }

    fn load(c: &mut Cpu, _d: Option<&Debugger>, _address: u16) -> Result<u8, CpuError> {
        Ok(c.regs.a)
    }
    fn store(c: &mut Cpu, _d: Option<&Debugger>, _address: u16, b: u8) -> Result<(), CpuError> {
        c.regs.a = b;
        Ok(())
    }
}

/**
 * Absolute addressing specifies the memory location explicitly in the two bytes following the opcode. So JMP $4032 will set the PC to $4032.
 * The hex for this is 4C 32 40. The 6502 is a little endian machine, so any 16 bit (2 byte) value is stored with the LSB first. All instructions that use absolute addressing are 3 bytes.
 */
pub(crate) struct AbsoluteAddressing;
impl AddressingMode for AbsoluteAddressing {
    fn len() -> i8 {
        3
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let b3 = m.read_byte((c.regs.pc.wrapping_add(2)) as usize)?;
        let tgt = Self::target_address(c, false)?;
        Ok(format!(
            "${:04x}:\t{:02x} {:02x} {:02x}\t-->\t{} ${:04x}\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            b3,
            opcode_name.to_uppercase(),
            (((b3 as u16) << 8) | (b2 as u16)),
            AddressingModeId::Abs,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        let w = c.bus.get_memory().read_word_le((c.regs.pc + 1) as usize)?;

        Ok((w, false))
    }
}

/**
 * This addressing mode makes the target address by adding the contents of the X or Y register to an absolute address.
 */
pub(crate) struct AbsoluteXAddressing;
impl AddressingMode for AbsoluteXAddressing {
    fn len() -> i8 {
        3
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let b3 = m.read_byte((c.regs.pc.wrapping_add(2)) as usize)?;
        let tgt = Self::target_address(c, false)?;
        Ok(format!(
            "${:04x}:\t{:02x} {:02x} {:02x}\t-->\t{} ${:04x}, X\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            b3,
            opcode_name.to_uppercase(),
            (((b3 as u16) << 8) | (b2 as u16)),
            AddressingModeId::Abx,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        let ww = w.wrapping_add(c.regs.x as u16);

        // check for page crossing, in case we need to add a cycle
        if add_extra_cycle_on_page_crossing && is_page_cross(w, ww) {
            return Ok((ww, true));
        }

        Ok((ww, false))
    }
}

/**
 * This addressing mode makes the target address by adding the contents of the X or Y register to an absolute address.
 */
pub(crate) struct AbsoluteYAddressing;
impl AddressingMode for AbsoluteYAddressing {
    fn len() -> i8 {
        3
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let b3 = m.read_byte((c.regs.pc.wrapping_add(2)) as usize)?;
        let tgt = Self::target_address(c, false)?;
        Ok(format!(
            "${:04x}:\t{:02x} {:02x} {:02x}\t-->\t{} ${:04x}, Y\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            b3,
            opcode_name.to_uppercase(),
            (((b3 as u16) << 8) | (b2 as u16)),
            AddressingModeId::Aby,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        let ww = w.wrapping_add(c.regs.y as u16);

        // check for page crossing, in case we need to add a cycle
        if add_extra_cycle_on_page_crossing && is_page_cross(w, ww) {
            return Ok((ww, true));
        }

        Ok((ww, false))
    }
}

/**
 * These instructions have their data defined as the next byte after the opcode. ORA #$B2 will perform a logical (also called bitwise) of the value B2 with the accumulator.
 */
pub(crate) struct ImmediateAddressing;
impl AddressingMode for ImmediateAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;
        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} #${:02x}\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Imm,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        let w = c.regs.pc.wrapping_add(1);
        Ok((w as u16, false))
    }
}

/**
 * In an implied instruction, there's no operand (implied in the instruction itself).
 */
pub(crate) struct ImpliedAddressing;
impl AddressingMode for ImpliedAddressing {
    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let b = c.bus.get_memory().read_byte(c.regs.pc as usize)?;
        Ok(format!(
            "${:04x}:\t{:02x}\t\t-->\t{}\t\t[{}]",
            c.regs.pc,
            b,
            opcode_name.to_uppercase(),
            AddressingModeId::Imp
        ))
    }
}

/**
 * The JMP instruction is the only instruction that uses this addressing mode. It is a 3 byte instruction - the 2nd and 3rd bytes are an absolute address.
 * The set the PC to the address stored at that address. So maybe this would be clearer.
 */
pub(crate) struct IndirectAddressing;
impl AddressingMode for IndirectAddressing {
    fn len() -> i8 {
        3
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let b3 = m.read_byte((c.regs.pc.wrapping_add(2)) as usize)?;
        let tgt = Self::target_address(c, false)?;
        Ok(format!(
            "${:04x}:\t{:02x} {:02x} {:02x}\t-->\t{} (${:04x})\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            b3,
            opcode_name.to_uppercase(),
            (((b3 as u16) << 8) | (b2 as u16)),
            AddressingModeId::Ind,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address
        let w = c.bus.get_memory().read_word_le((c.regs.pc + 1) as usize)?;

        let ww: u16;
        if w & 0xff == 0xff && c.cpu_type == CpuType::MOS6502 {
            // emulate 6502 JMP bug on access across page boundary (this addressing mode is used by JMP only):
            // An original 6502 has does not correctly fetch the target address if the indirect vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
            // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
            let lsb = c.bus.get_memory().read_byte(w as usize)?;
            let msb = c.bus.get_memory().read_byte((w & 0xff00) as usize)?;
            ww = ((msb as u16) << 8) | (lsb as u16);
        } else {
            // read word at address
            ww = c.bus.get_memory().read_word_le(w as usize)?;
        }

        Ok((ww, false))
    }
}

/**
 * This mode is only used with the X register.
 * Consider a situation where the instruction is LDA ($20,X), X contains $04, and memory at $24 contains 0024: 74 20, First, X is added to $20 to get $24.
 * The target address will be fetched from $24 resulting in a target address of $2074. Register A will be loaded with the contents of memory at $2074.
 *
 * If X + the immediate byte will wrap around to a zero-page address. So you could code that like targetAddress = (X + opcode[1]) & 0xFF .
 * Indexed Indirect instructions are 2 bytes - the second byte is the zero-page address - $20 in the example. Obviously the fetched address has to be stored in the zero page.
 */
pub(crate) struct XIndirectAddressing;
impl AddressingMode for XIndirectAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} (${:02x}, X)\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Xin,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address in zeropage
        let mut w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // add x (wrapping), and read word
        w = w.wrapping_add(c.regs.x);
        let ww = c.bus.get_memory().read_word_le(w as usize)?;

        Ok((ww, false))
    }
}

/**
 * This mode is only used with the Y register. It differs in the order that Y is applied to the indirectly fetched address.
 *
 * An example instruction that uses indirect index addressing is LDA ($86),Y . To calculate the target address, the CPU will first fetch the address stored at zero page location $86.
 * That address will be added to register Y to get the final target address. For LDA ($86),Y, if the address stored at $86 is $4028 (memory is 0086: 28 40, remember little endian) and
 * register Y contains $10, then the final target address would be $4038. Register A will be loaded with the contents of memory at $4038.
 *
 * Indirect Indexed instructions are 2 bytes - the second byte is the zero-page address - $86 in the example. (So the fetched address has to be stored in the zero page.)
 *
 * While indexed indirect addressing will only generate a zero-page address, this mode's target address is not wrapped - it can be anywhere in the 16-bit address space.
 */
pub(crate) struct IndirectYAddressing;
impl AddressingMode for IndirectYAddressing {
    fn len() -> i8 {
        2
    }
    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} (${:02x}), Y\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2 as u8,
            AddressingModeId::Iny,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address contained at address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let ww = c.bus.get_memory().read_word_le(w as usize)?;

        // add y
        let addr_plus_y = ww.wrapping_add(c.regs.y as u16);

        // check for page crossing, in case we need to add a cycle
        if add_extra_cycle_on_page_crossing && is_page_cross(ww, addr_plus_y) {
            return Ok((addr_plus_y, true));
        }

        Ok((addr_plus_y, false))
    }
}

/**
 * Relative addressing on the 6502 is only used for branch operations. The byte after the opcode is the branch offset.
 * If the branch is taken, the new address will the the current PC plus the offset.
 * The offset is a signed byte, so it can jump a maximum of 127 bytes forward, or 128 bytes backward.
 */
pub(crate) struct RelativeAddressing;
impl AddressingMode for RelativeAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} ${:02x}\t\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Rel,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        let w = c.regs.pc.wrapping_add(1);

        // this will check for page crossing too (check mandatory in relative addressing)
        let (_, cross) =
            get_relative_branch_target(c.regs.pc, c.bus.get_memory().read_byte(w as usize)?);
        Ok((w as u16, cross))
    }
}

/**
 * Zero-Page is an addressing mode that is only capable of addressing the first 256 bytes of the CPU's memory map. You can think of it as absolute addressing for the first 256 bytes.
 * The instruction LDA $35 will put the value stored in memory location $35 into A.
 * The advantage of zero-page are two - the instruction takes one less byte to specify, and it executes in less CPU cycles.
 * Most programs are written to store the most frequently used variables in the first 256 memory locations so they can take advantage of zero page addressing.
 */
pub(crate) struct ZeroPageAddressing;
impl AddressingMode for ZeroPageAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} ${:02x}\t\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Zpg,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        Ok((w as u16, false))
    }
}

/**
 * This works just like absolute indexed, but the target address is limited to the first 0xFF bytes.
 * The target address will wrap around and will always be in the zero page. If the instruction is LDA $C0,X, and X is $60, then the target address will be $20.
 * $C0+$60 = $120, but the carry is discarded in the calculation of the target address.
 */
pub(crate) struct ZeroPageXAddressing;
impl AddressingMode for ZeroPageXAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} ${:02x}, X\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Zpx,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // and add x, wrapping
        let w = w.wrapping_add(c.regs.x);
        Ok((w as u16, false))
    }
}

/**
 * This works just like absolute indexed, but the target address is limited to the first 0xFF bytes.
 * The target address will wrap around and will always be in the zero page. If the instruction is LDA $C0,X, and X is $60, then the target address will be $20.
 * $C0+$60 = $120, but the carry is discarded in the calculation of the target address.
 */
pub(crate) struct ZeroPageYAddressing;
impl AddressingMode for ZeroPageYAddressing {
    fn len() -> i8 {
        2
    }

    fn repr(c: &mut Cpu, opcode_name: &str) -> Result<String, CpuError> {
        let m = c.bus.get_memory();
        let b1 = m.read_byte(c.regs.pc as usize)?;
        let b2 = m.read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        let tgt = Self::target_address(c, false)?;

        Ok(format!(
            "${:04x}:\t{:02x} {:02x}\t\t-->\t{} ${:02x}, Y\t[{}, tgt=${:04x}]",
            c.regs.pc,
            b1,
            b2,
            opcode_name.to_uppercase(),
            b2,
            AddressingModeId::Zpy,
            tgt.0
        ))
    }

    fn target_address(
        c: &mut Cpu,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, bool), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // and add y, wrapping
        let w = w.wrapping_add(c.regs.y);
        Ok((w as u16, false))
    }
}
