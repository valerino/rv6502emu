/*
 * Filename: /src/cpu/opcodes.rs
 * Project: rv6502emu
 * Created Date: Thursday, January 1st 1970, 1:00:00 am
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

use crate::cpu::addressing_modes::*;
use crate::cpu::Cpu;
use crate::memory::memory_error::MemoryError;
use crate::memory::Memory;
use lazy_static::*;
use log::*;

lazy_static! {
    /**
     * the 256 opcodes table
     *
     * each opcode gets in input a reference to the Cpu and the cycles needed to execute the opcode, then returns the effective elapsed cycles (may include an
     * additional cycle due to page boundaries crossing).
     */
    pub static ref OPCODE_MATRIX: Vec<(fn(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError>, usize)> =
        vec![(adc::<AccumulatorAddressing>, 1), (adc::<AccumulatorAddressing>, 3)];
}

fn adc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn and<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn asl<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bcc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bcs<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn beq<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bit<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bmi<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bne<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bpl<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn brk<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bvc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn bvs<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn clc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn cld<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn cli<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn clv<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn cmp<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn cpx<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn cpy<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn dec<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn dex<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn dey<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn eor<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn inc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn inx<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn iny<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn jmp<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn jsr<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn kil<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    // panic!
    // TODO: handle debug
    panic!("KIL\n--> {}", c.regs);
}

fn lda<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn ldx<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn ldy<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn lsr<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn nop<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn ora<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn pha<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn php<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn pla<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn plp<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn rol<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn ror<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn rti<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn rts<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sbc<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sec<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sed<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sei<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sta<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn stx<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn sty<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn tax<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn tay<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn tsx<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn txa<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn txs<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}

fn tya<a: AddressingMode>(c: &mut Cpu, in_cycles: usize) -> Result<usize, MemoryError> {
    a::operand(c);
    Ok((in_cycles))
}
