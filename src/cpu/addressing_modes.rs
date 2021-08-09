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
     * fetch the operand (the target address)
     */
    fn operand(_c: &Cpu) -> Result<u16, MemoryError> {
        Ok(0)
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

pub struct AccumulatorAddressing;

impl AddressingMode for AccumulatorAddressing {
    fn operand(_c: &Cpu) -> Result<u16, MemoryError> {
        // implied A
        Ok(0)
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
    fn operand(_c: &Cpu) -> Result<u16, MemoryError> {
        // implied A
        Ok(0)
    }

    fn load(c: &mut Cpu, address: u16) -> Result<u8, MemoryError> {
        Ok(c.regs.a)
    }
    fn store(c: &mut Cpu, _: u16, b: u8) -> Result<(), MemoryError> {
        c.regs.a = b;
        Ok(())
    }
}

pub struct AbsoluteXAddressing;
impl AddressingMode for AbsoluteXAddressing {}

pub struct AbsoluteYAddressing;
impl AddressingMode for AbsoluteYAddressing {}

pub struct ImmediateAddressing;
impl AddressingMode for ImmediateAddressing {}

pub struct ImpliedAddressing;
impl AddressingMode for ImpliedAddressing {}

pub struct IndirectAddressing;
impl AddressingMode for IndirectAddressing {}

pub struct XIndirectAddressing;
impl AddressingMode for XIndirectAddressing {}

pub struct IndirectYAddressing;
impl AddressingMode for IndirectYAddressing {}

pub struct RelativeAddressing;
impl AddressingMode for RelativeAddressing {}

pub struct ZeroPageAddressing;
impl AddressingMode for ZeroPageAddressing {}

pub struct ZeroPageXAddressing;
impl AddressingMode for ZeroPageXAddressing {}

pub struct ZeroPageYAddressing;
impl AddressingMode for ZeroPageYAddressing {}
