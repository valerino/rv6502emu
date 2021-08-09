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

use crate::cpu::Cpu;
use crate::cpu::CpuCallbackContext;
use crate::cpu::CpuOperation;
use crate::memory::memory_error::MemoryError;
use log::*;

/**
 * http://www.emulator101.com/6502-addressing-modes.html
 * https://www.masswerk.at/6502/6502_instruction_set.html
 */
pub trait AddressingMode {
    /**
     * the instruction size
     */
    fn len() -> u16 {
        1
    }

    /**
     * string representation
     */
    fn repr(opcode_name: &str, _operand: u16) -> String {
        String::from(opcode_name)
    }

    /**
     * fetch the operand (the target address), returns a tuple with (address, extra_cycle_if_page_crossed))
     */
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        Ok((0, false))
    }

    /**
     * load byte from address
     */
    fn load(c: &mut Cpu, address: u16) -> Result<u8, MemoryError> {
        let m = c.bus.get_memory();
        let b = m.read_byte(address as usize)?;

        // call callback
        Ok(b)
    }

    /**
     * store byte to address
     */
    fn store(c: &mut Cpu, address: u16, b: u8) -> Result<(), MemoryError> {
        let m = c.bus.get_memory();
        let res = m.write_byte(address as usize, b);
        res
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

pub struct AccumulatorAddressing;
impl AddressingMode for AccumulatorAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} A", opcode_name)
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        // implied A
        Ok((0, false))
    }

    fn load(c: &mut Cpu, _: u16) -> Result<u8, MemoryError> {
        Ok(c.regs.a)
    }
    fn store(c: &mut Cpu, _: u16, b: u8) -> Result<(), MemoryError> {
        c.regs.a = b;
        Ok(())
    }
}

pub struct AbsoluteAddressing;
impl AddressingMode for AbsoluteAddressing {
    fn len() -> u16 {
        3
    }

    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:02x}", opcode_name, _operand)
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c
            .bus
            .get_memory()
            .read_word_le((_c.regs.pc + 1) as usize)?;

        Ok((w, false))
    }
}

pub struct AbsoluteXAddressing;
impl AddressingMode for AbsoluteXAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:02x}, X", opcode_name, _operand)
    }

    fn len() -> u16 {
        3
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c
            .bus
            .get_memory()
            .read_word_le((_c.regs.pc + 1) as usize)?;
        let ww = w.wrapping_add(_c.regs.x as u16);

        if is_page_cross(w, ww) {
            return Ok((ww, true));
        }

        Ok((ww, false))
    }
}

pub struct AbsoluteYAddressing;
impl AddressingMode for AbsoluteYAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:02x}, Y", opcode_name, _operand)
    }

    fn len() -> u16 {
        3
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c
            .bus
            .get_memory()
            .read_word_le((_c.regs.pc + 1) as usize)?;
        let ww = w.wrapping_add(_c.regs.y as u16);

        if is_page_cross(w, ww) {
            return Ok((ww, true));
        }

        Ok((ww, false))
    }
}

pub struct ImmediateAddressing;
impl AddressingMode for ImmediateAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} #${:x}", opcode_name, _operand as u8)
    }

    fn len() -> u16 {
        2
    }
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;
        Ok((w as u16, false))
    }
}

pub struct ImpliedAddressing;
impl AddressingMode for ImpliedAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{}", opcode_name)
    }
}

pub struct IndirectAddressing;
impl AddressingMode for IndirectAddressing {
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} (${:02x})", opcode_name, _operand)
    }

    fn len() -> u16 {
        3
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        // read address
        let w = _c
            .bus
            .get_memory()
            .read_word_le((_c.regs.pc + 1) as usize)?;

        // read word at address
        let ww = _c.bus.get_memory().read_word_le(w as usize)?;

        Ok((ww, false))
    }
}

pub struct XIndirectAddressing;
impl AddressingMode for XIndirectAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} (${:x}, X)", opcode_name, _operand as u8)
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        // read address in zeropage
        let mut w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;

        // add x, and read word
        w = w.wrapping_add(_c.regs.x);
        let ww = _c.bus.get_memory().read_word_le(w as usize)?;

        Ok((ww, false))
    }
}

pub struct IndirectYAddressing;
impl AddressingMode for IndirectYAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} (${:x}), Y", opcode_name, _operand as u8)
    }

    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        // read address in zeropage
        let mut w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;

        // read word at address
        let mut ww = _c.bus.get_memory().read_word_le(w as usize)?;

        // read word at [address + y]
        ww = ww.wrapping_add(_c.regs.y as u16);
        let www = _c.bus.get_memory().read_word_le(ww as usize)?;

        Ok((www, false))
    }
}

pub struct RelativeAddressing;
impl AddressingMode for RelativeAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:x}", opcode_name, _operand as u8)
    }
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        // this is the offset to be added (signed) to PC
        let w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;
        Ok((w as u16, false))
    }
}

pub struct ZeroPageAddressing;
impl AddressingMode for ZeroPageAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:x}", opcode_name, _operand as u8)
    }
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;
        Ok((w as u16, false))
    }
}

pub struct ZeroPageXAddressing;
impl AddressingMode for ZeroPageXAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:x}, X", opcode_name, _operand as u8)
    }
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;
        let ww = w.wrapping_add(_c.regs.x as u8);

        let www = _c.bus.get_memory().read_word_le(ww as usize)?;
        Ok((www as u16, false))
    }
}

pub struct ZeroPageYAddressing;
impl AddressingMode for ZeroPageYAddressing {
    fn len() -> u16 {
        2
    }
    fn repr(opcode_name: &str, _operand: u16) -> String {
        format!("{} ${:x}, Y", opcode_name, _operand as u8)
    }
    fn operand(_c: &mut Cpu) -> Result<(u16, bool), MemoryError> {
        let w = _c.bus.get_memory().read_byte((_c.regs.pc + 1) as usize)?;
        let mut ww = _c.bus.get_memory().read_word_le(w as usize)?;
        ww = ww.wrapping_add(_c.regs.y as u16);
        let www = _c.bus.get_memory().read_word_le(ww as usize)?;
        Ok((www as u16, false))
    }
}
