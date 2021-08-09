/*
 * Filename: /src/cpu/opcodes.rs
 * Project: rv6502emu
 * Created Date: 2021-08-09, 12:52:20
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
     * each opcode gets in input a reference to the Cpu, the cycles needed to execute the opcode, and a boolean to indicate if, on crossing page boundaries, an extra cycles must be added.
     * returns the effective elapsed cycles (may include the aferomentioned additional cycle).
     */
    pub static ref OPCODE_MATRIX: Vec<(fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool) -> Result<usize, MemoryError>, usize, bool)> =
        vec![
            /// 0x0 - 0xf
            (brk::<ImpliedAddressing>, 7, false), (ora::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (slo::<XIndirectAddressing>, 8, false),
            (nop::<ZeroPageAddressing>, 3, false), (ora::<ZeroPageAddressing>, 3, false), (asl::<ZeroPageAddressing>, 5, false), (slo::<ZeroPageAddressing>, 5, false),
            (php::<ImpliedAddressing>, 3, false), (ora::<ImmediateAddressing>, 2, false), (asl::<AccumulatorAddressing>, 2, false), (anc::<ImmediateAddressing>, 2, false),
            (nop::<AbsoluteAddressing>, 4, false), (ora::<AbsoluteAddressing>, 4, false), (asl::<AbsoluteAddressing>, 6, false), (slo::<AbsoluteAddressing>, 6, false),

            /// 0x10 - 0x1f
            (bpl::<RelativeAddressing>, 2, true), (ora::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (slo::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (ora::<ZeroPageXAddressing>, 4, false), (asl::<ZeroPageXAddressing>, 6, false), (slo::<ZeroPageXAddressing>, 6, false),
            (clc::<ImpliedAddressing>, 2, false), (ora::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (slo::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (ora::<AbsoluteXAddressing>, 4, true), (asl::<AbsoluteXAddressing>, 7, false), (slo::<AbsoluteXAddressing>, 7, false),

            /// 0x20 - 0x2f
            (jsr::<AbsoluteAddressing>, 6, false), (and::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (rla::<XIndirectAddressing>, 8, false),
            (bit::<ZeroPageAddressing>, 3, false), (and::<ZeroPageAddressing>, 3, false), (rol::<ZeroPageAddressing>, 5, false), (rla::<ZeroPageAddressing>, 5, false),
            (plp::<ImpliedAddressing>, 4, false), (and::<ImmediateAddressing>, 2, false), (rol::<AccumulatorAddressing>, 2, false), (anc::<ImmediateAddressing>, 2, false),
            (bit::<AbsoluteAddressing>, 4, false), (and::<AbsoluteAddressing>, 4, false), (rol::<AbsoluteAddressing>, 6, false), (rla::<AbsoluteAddressing>, 6, false),

            /// 0x30 - 0x3f
            (bmi::<RelativeAddressing>, 2, true), (and::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (rla::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (and::<ZeroPageXAddressing>, 4, false), (rol::<ZeroPageXAddressing>, 6, false), (rla::<ZeroPageXAddressing>, 6, false),
            (sec::<ImpliedAddressing>, 2, false), (and::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (rla::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (and::<AbsoluteXAddressing>, 4, true), (rol::<AbsoluteXAddressing>, 7, false), (rla::<AbsoluteXAddressing>, 7, false),

            /// 0x40 - 0x4f
            (rti::<ImpliedAddressing>, 6, false), (eor::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (sre::<XIndirectAddressing>, 8, false),
            (nop::<ZeroPageAddressing>, 3, false), (eor::<ZeroPageAddressing>, 3, false), (lsr::<ZeroPageAddressing>, 5, false), (sre::<ZeroPageAddressing>, 5, false),
            (pha::<ImpliedAddressing>, 3, false), (eor::<ImmediateAddressing>, 2, false), (lsr::<AccumulatorAddressing>, 2, false), (alr::<ImmediateAddressing>, 2, false),
            (jmp::<AbsoluteAddressing>, 3, false), (eor::<AbsoluteAddressing>, 4, false), (lsr::<AbsoluteAddressing>, 6, false), (sre::<AbsoluteAddressing>, 6, false),
            ];
}

fn adc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn alr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn anc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn and<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn asl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bcc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bcs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn beq<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bit<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bmi<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bne<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bpl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn brk<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bvc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn bvs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn clc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn cld<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn cli<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn clv<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn cmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn cpx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn cpy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn dec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn dex<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn dey<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn eor<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn inc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn inx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn iny<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn jmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn jsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn kil<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    // panic!
    // TODO: handle debug
    panic!("KIL\n--> {}", c.regs);
}

fn lda<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn ldx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn ldy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn lsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn nop<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn ora<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn pha<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn php<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn pla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn plp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn rla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn rol<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn ror<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn rti<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn rts<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sbc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sed<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sei<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn slo<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sre<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sta<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn stx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn sty<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn tax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn tay<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn tsx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn txa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn txs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}

fn tya<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<usize, MemoryError> {
    A::operand(c);
    Ok((in_cycles))
}
