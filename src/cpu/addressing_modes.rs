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
    Aix,
    Imm,
    Imp,
    Ind,
    Izp,
    Xin,
    Iny,
    Rel,
    Zpg,
    Zpx,
    Zpy,
    Zpr,
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
            AddressingModeId::Aix => {
                write!(f, "AiX")?;
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
            AddressingModeId::Izp => {
                write!(f, "Izp")?;
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
            AddressingModeId::Zpr => {
                write!(f, "Zpr")?;
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
     * the addressing mode.
     */
    fn id() -> AddressingModeId;

    /**
     * instruction size.
     */
    fn len() -> i8;

    /**
     * the operand at PC+1.
     */
    fn operand(c: &mut Cpu) -> Result<u16, CpuError>;

    /**
     * fetch the opcode target depending on the addressing mode, returns a tuple with (address, effective cycles including page crossing))
     */
    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError>;

    /**
     * load byte from address
     */
    fn load(c: &mut Cpu, address: u16) -> Result<u8, CpuError> {
        let m = c.bus.get_memory();

        // read
        let b = m.read_byte(address as usize)?;

        /*
        // check if a breakpoint has to be triggered
        if d.is_some() {
            d.unwrap()
                .handle_rw_breakpoint(c, address, BreakpointType::READ)?
        }
        */
        // call callback if any
        c.call_callback(address, b, 1, CpuOperation::Read);
        Ok(b)
    }

    /**
     * store byte to address
     */
    fn store(c: &mut Cpu, address: u16, b: u8) -> Result<(), CpuError> {
        let m = c.bus.get_memory();

        // write
        m.write_byte(address as usize, b)?;

        /*
        // check if a breakpoint has to be triggered
        if d.is_some() {
            d.unwrap()
                .handle_rw_breakpoint(c, address, BreakpointType::WRITE)?
        }
        */
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
    fn id() -> AddressingModeId {
        AddressingModeId::Acc
    }

    fn len() -> i8 {
        1
    }
    fn operand(_c: &mut Cpu) -> Result<u16, CpuError> {
        // no operand, implied (A)
        Ok(0)
    }
    fn target(
        _c: &mut Cpu,
        in_cycles: usize,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        Ok((0, in_cycles))
    }

    fn load(c: &mut Cpu, __address: u16) -> Result<u8, CpuError> {
        Ok(c.regs.a)
    }
    fn store(c: &mut Cpu, __address: u16, b: u8) -> Result<(), CpuError> {
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
    fn id() -> AddressingModeId {
        AddressingModeId::Abs
    }

    fn len() -> i8 {
        3
    }
    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        _add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = Self::operand(c)?;
        Ok((w, in_cycles))
    }
}

/**
 * This addressing mode makes the target address by adding the contents of the X or Y register to an absolute address.
 */
pub(crate) struct AbsoluteXAddressing;
impl AddressingMode for AbsoluteXAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Abx
    }

    fn len() -> i8 {
        3
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = Self::operand(c)?;
        let ww = w.wrapping_add(c.regs.x as u16);

        // check for page crossing, in case we need to add a cycle
        if add_extra_cycle_on_page_crossing && is_page_cross(w, ww) {
            return Ok((ww, in_cycles + 1));
        }

        Ok((ww, in_cycles))
    }
}

/**
 * This addressing mode makes the target address by adding the contents of the X or Y register to an absolute address.
 */
pub(crate) struct AbsoluteYAddressing;
impl AddressingMode for AbsoluteYAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Aby
    }

    fn len() -> i8 {
        3
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = Self::operand(c)?;
        let ww = w.wrapping_add(c.regs.y as u16);

        // check for page crossing, in case we need to add a cycle
        if add_extra_cycle_on_page_crossing && is_page_cross(w, ww) {
            return Ok((ww, in_cycles + 1));
        }

        Ok((ww, in_cycles))
    }
}

/**
 * These instructions have their data defined as the next byte after the opcode. ORA #$B2 will perform a logical (also called bitwise) of the value B2 with the accumulator.
 */
pub(crate) struct ImmediateAddressing;
impl AddressingMode for ImmediateAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Imm
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = c.regs.pc.wrapping_add(1);
        Ok((w, in_cycles))
    }
}

/**
 * In an implied instruction, there's no operand (implied in the instruction itself).
 */
pub(crate) struct ImpliedAddressing;
impl AddressingMode for ImpliedAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Imp
    }

    fn len() -> i8 {
        1
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        Ok(0)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        Ok((0, in_cycles))
    }
}

/**
 * The JMP instruction is the only instruction that uses this addressing mode. It is a 3 byte instruction - the 2nd and 3rd bytes are an absolute address.
 * The set the PC to the address stored at that address. So maybe this would be clearer.
 */
pub(crate) struct IndirectAddressing;
impl AddressingMode for IndirectAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Ind
    }

    fn len() -> i8 {
        3
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
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

        Ok((ww, in_cycles))
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
    fn id() -> AddressingModeId {
        AddressingModeId::Xin
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // read address in zeropage
        let mut w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // add x (wrapping), and read word
        w = w.wrapping_add(c.regs.x);
        let ww = c.bus.get_memory().read_word_le(w as usize)?;

        Ok((ww, in_cycles))
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
    fn id() -> AddressingModeId {
        AddressingModeId::Iny
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
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
            return Ok((addr_plus_y, in_cycles + 1));
        }

        Ok((addr_plus_y, in_cycles))
    }
}

/**
 * Relative addressing on the 6502 is only used for branch operations. The byte after the opcode is the branch offset.
 * If the branch is taken, the new address will the the current PC plus the offset.
 * The offset is a signed byte, so it can jump a maximum of 127 bytes forward, or 128 bytes backward.
 */
pub(crate) struct RelativeAddressing;
impl AddressingMode for RelativeAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Rel
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = c.regs.pc.wrapping_add(1);

        // this will check for page crossing too (check mandatory in relative addressing)
        let (_, cross) =
            get_relative_branch_target(c.regs.pc, c.bus.get_memory().read_byte(w as usize)?);
        Ok((w as u16, in_cycles + (cross as usize)))
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
    fn id() -> AddressingModeId {
        AddressingModeId::Zpg
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        Ok((w as u16, in_cycles))
    }
}

/**
 * This works just like absolute indexed, but the target address is limited to the first 0xFF bytes.
 * The target address will wrap around and will always be in the zero page. If the instruction is LDA $C0,X, and X is $60, then the target address will be $20.
 * $C0+$60 = $120, but the carry is discarded in the calculation of the target address.
 */
pub(crate) struct ZeroPageXAddressing;
impl AddressingMode for ZeroPageXAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Zpx
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // and add x, wrapping
        let w = w.wrapping_add(c.regs.x);
        Ok((w as u16, in_cycles))
    }
}

/**
 * This works just like absolute indexed, but the target address is limited to the first 0xFF bytes.
 * The target address will wrap around and will always be in the zero page. If the instruction is LDA $C0,X, and X is $60, then the target address will be $20.
 * $C0+$60 = $120, but the carry is discarded in the calculation of the target address.
 */
pub(crate) struct ZeroPageYAddressing;
impl AddressingMode for ZeroPageYAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Zpy
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // and add y, wrapping
        let w = w.wrapping_add(c.regs.y);
        Ok((w as u16, in_cycles))
    }
}

/**
 * 65C02 only!
 * Many 65C02 instruction can operate on memory locations specified indirectly through zero page.
 * For example if location $20 contains $31 and location $21 contains $65 then the instruction LDA ($20) will load the byte stored at $6531 into the accumulator.
 */
pub(crate) struct IndirectZeroPageAddressing;
impl AddressingMode for IndirectZeroPageAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Izp
    }

    fn len() -> i8 {
        2
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w as u16)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // read address in the zeropage
        let w = c
            .bus
            .get_memory()
            .read_byte((c.regs.pc.wrapping_add(1)) as usize)?;

        // read address indirect
        let ww = c.bus.get_memory().read_word_le(w as usize)?;
        Ok((ww as u16, in_cycles))
    }
}

/**
 * 65C02 only!
 * This mode has been added to the JMP instruction to support jump tables.
 * The value of the X register is added to the absolute location in the instruction to form the address.
 * The 16 bit value held at this address is the final target location.
 */
pub(crate) struct AbsoluteIndirectXAddressing;
impl AddressingMode for AbsoluteIndirectXAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Aix
    }

    fn len() -> i8 {
        3
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        let ww = w.wrapping_add(c.regs.x as u16);
        let www = c.bus.get_memory().read_word_le(ww as usize)?;
        Ok((www, in_cycles))
    }
}

/**
 * 65C02 only!
 * This is mostly the same as Relative addressing.
 * addr = rd_mem16(pc+1) + pc + 2;  pc += 0; // Leave PC at ZP operand
 */
pub(crate) struct ZeroPageRelativeAddressing;
impl AddressingMode for ZeroPageRelativeAddressing {
    fn id() -> AddressingModeId {
        AddressingModeId::Zpr
    }

    fn len() -> i8 {
        3
    }

    fn operand(c: &mut Cpu) -> Result<u16, CpuError> {
        // this must be treated as
        // hi=zeropage address
        // lo=offset
        let w = c
            .bus
            .get_memory()
            .read_word_le((c.regs.pc.wrapping_add(1)) as usize)?;
        Ok(w)
    }

    fn target(
        c: &mut Cpu,
        in_cycles: usize,
        add_extra_cycle_on_page_crossing: bool,
    ) -> Result<(u16, usize), CpuError> {
        // pc+1=byte to test
        // pc+2=pc-relative offset to branch to
        let w = c.regs.pc.wrapping_add(2);
        Ok((w as u16, in_cycles))
    }
}
