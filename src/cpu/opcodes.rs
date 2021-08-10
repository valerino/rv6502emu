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
use crate::cpu::CpuFlags;
use crate::memory::memory_error::MemoryError;
use crate::memory::Memory;
use ::function_name::named;
use lazy_static::*;
use log::*;

lazy_static! {
    /**
     * the 256 opcodes table
     *
     * each opcode gets in input a reference to the Cpu, the cycles needed to execute the opcode, and a boolean to indicate if, on crossing page boundaries, an extra cycles must be added.
     * returns a tuple with the instruction size and the effective elapsed cycles (may include the aferomentioned additional cycle).
     */
    pub static ref OPCODE_MATRIX: Vec<(fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool) -> Result<(u16, usize), MemoryError>, usize, bool)> =
        vec![
            // 0x0 - 0xf
            (brk::<ImpliedAddressing>, 7, false), (ora::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (slo::<XIndirectAddressing>, 8, false),
            (nop::<ZeroPageAddressing>, 3, false), (ora::<ZeroPageAddressing>, 3, false), (asl::<ZeroPageAddressing>, 5, false), (slo::<ZeroPageAddressing>, 5, false),
            (php::<ImpliedAddressing>, 3, false), (ora::<ImmediateAddressing>, 2, false), (asl::<AccumulatorAddressing>, 2, false), (anc::<ImmediateAddressing>, 2, false),
            (nop::<AbsoluteAddressing>, 4, false), (ora::<AbsoluteAddressing>, 4, false), (asl::<AbsoluteAddressing>, 6, false), (slo::<AbsoluteAddressing>, 6, false),

            // 0x10 - 0x1f
            (bpl::<RelativeAddressing>, 2, true), (ora::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (slo::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (ora::<ZeroPageXAddressing>, 4, false), (asl::<ZeroPageXAddressing>, 6, false), (slo::<ZeroPageXAddressing>, 6, false),
            (clc::<ImpliedAddressing>, 2, false), (ora::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (slo::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (ora::<AbsoluteXAddressing>, 4, true), (asl::<AbsoluteXAddressing>, 7, false), (slo::<AbsoluteXAddressing>, 7, false),

            // 0x20 - 0x2f
            (jsr::<AbsoluteAddressing>, 6, false), (and::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (rla::<XIndirectAddressing>, 8, false),
            (bit::<ZeroPageAddressing>, 3, false), (and::<ZeroPageAddressing>, 3, false), (rol::<ZeroPageAddressing>, 5, false), (rla::<ZeroPageAddressing>, 5, false),
            (plp::<ImpliedAddressing>, 4, false), (and::<ImmediateAddressing>, 2, false), (rol::<AccumulatorAddressing>, 2, false), (anc::<ImmediateAddressing>, 2, false),
            (bit::<AbsoluteAddressing>, 4, false), (and::<AbsoluteAddressing>, 4, false), (rol::<AbsoluteAddressing>, 6, false), (rla::<AbsoluteAddressing>, 6, false),

            // 0x30 - 0x3f
            (bmi::<RelativeAddressing>, 2, true), (and::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (rla::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (and::<ZeroPageXAddressing>, 4, false), (rol::<ZeroPageXAddressing>, 6, false), (rla::<ZeroPageXAddressing>, 6, false),
            (sec::<ImpliedAddressing>, 2, false), (and::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (rla::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (and::<AbsoluteXAddressing>, 4, true), (rol::<AbsoluteXAddressing>, 7, false), (rla::<AbsoluteXAddressing>, 7, false),

            // 0x40 - 0x4f
            (rti::<ImpliedAddressing>, 6, false), (eor::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (sre::<XIndirectAddressing>, 8, false),
            (nop::<ZeroPageAddressing>, 3, false), (eor::<ZeroPageAddressing>, 3, false), (lsr::<ZeroPageAddressing>, 5, false), (sre::<ZeroPageAddressing>, 5, false),
            (pha::<ImpliedAddressing>, 3, false), (eor::<ImmediateAddressing>, 2, false), (lsr::<AccumulatorAddressing>, 2, false), (alr::<ImmediateAddressing>, 2, false),
            (jmp::<AbsoluteAddressing>, 3, false), (eor::<AbsoluteAddressing>, 4, false), (lsr::<AbsoluteAddressing>, 6, false), (sre::<AbsoluteAddressing>, 6, false),

            // 0x50 - 0x5f
            (bvc::<ImpliedAddressing>, 2, true), (eor::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (sre::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (eor::<ZeroPageXAddressing>, 4, false), (lsr::<ZeroPageXAddressing>, 6, false), (sre::<ZeroPageXAddressing>, 6, false),
            (cli::<ImpliedAddressing>, 2, false), (eor::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (sre::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (eor::<AbsoluteXAddressing>, 4, true), (lsr::<AbsoluteXAddressing>, 7, false), (sre::<AbsoluteXAddressing>, 7, false),

            // 0x60 - 0x6f
            (rts::<RelativeAddressing>, 6, false), (adc::<XIndirectAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (rra::<XIndirectAddressing>, 8, false),
            (nop::<ZeroPageAddressing>, 3, false), (adc::<ZeroPageAddressing>, 3, false), (ror::<ZeroPageAddressing>, 5, false), (rra::<ZeroPageAddressing>, 5, false),
            (pla::<ImpliedAddressing>, 4, false), (adc::<ImmediateAddressing>, 2, true), (ror::<AccumulatorAddressing>, 2, false), (arr::<ImmediateAddressing>, 2, false),
            (jmp::<IndirectAddressing>, 5, false), (adc::<AbsoluteAddressing>, 4, false), (ror::<AbsoluteAddressing>, 6, false), (rra::<AbsoluteAddressing>, 6, false),

            // 0x70 - 0x7f
            (bvs::<RelativeAddressing>, 2, true), (adc::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (rra::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (adc::<ZeroPageXAddressing>, 4, false), (ror::<ZeroPageXAddressing>, 6, false), (rra::<ZeroPageXAddressing>, 6, false),
            (sei::<ImpliedAddressing>, 2, false), (adc::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (rra::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (adc::<AbsoluteXAddressing>, 4, true), (ror::<AbsoluteXAddressing>, 7, false), (rra::<AbsoluteXAddressing>, 7, false),

            // 0x80 - 0x8f
            (nop::<ImmediateAddressing>, 2, false), (sta::<XIndirectAddressing>, 6, false), (nop::<ImmediateAddressing>, 2, false), (sax::<XIndirectAddressing>, 6, false),
            (sty::<ZeroPageAddressing>, 3, false), (sta::<ZeroPageAddressing>, 3, false), (stx::<ZeroPageAddressing>, 3, false), (sax::<ZeroPageAddressing>, 3, false),
            (dey::<ImpliedAddressing>, 2, false), (nop::<ImmediateAddressing>, 2, false), (txa::<ImpliedAddressing>, 2, false), (xaa::<ImmediateAddressing>, 2, false),
            (sty::<AbsoluteAddressing>, 4, false), (sta::<AbsoluteAddressing>, 4, false), (stx::<AbsoluteAddressing>, 4, false), (sax::<AbsoluteAddressing>, 4, false),

            // 0x90 - 0x9f
            (bcc::<RelativeAddressing>, 2, true), (sta::<IndirectYAddressing>, 6, false), (kil::<ImpliedAddressing>, 0, false), (ahx::<IndirectYAddressing>, 6, false),
            (sty::<ZeroPageXAddressing>, 4, false), (sta::<ZeroPageXAddressing>, 4, false), (stx::<ZeroPageYAddressing>, 4, false), (sax::<ZeroPageYAddressing>, 4, false),
            (tya::<ImpliedAddressing>, 2, false), (sta::<AbsoluteYAddressing>, 5, false), (txs::<ImpliedAddressing>, 2, false), (tas::<AbsoluteYAddressing>, 5, false),
            (shy::<AbsoluteXAddressing>, 5, false), (sta::<AbsoluteXAddressing>, 5, false), (shx::<AbsoluteYAddressing>, 5, false), (ahx::<AbsoluteYAddressing>, 5, false),

            // 0xa0 - 0xaf
            (ldy::<ImmediateAddressing>, 2, false), (lda::<XIndirectAddressing>, 6, false), (ldx::<ImmediateAddressing>, 2, false), (lax::<XIndirectAddressing>, 6, false),
            (ldy::<ZeroPageAddressing>, 3, false), (lda::<ZeroPageAddressing>, 3, false), (ldx::<ZeroPageAddressing>, 3, false), (lax::<ZeroPageAddressing>, 3, false),
            (tay::<ImpliedAddressing>, 2, false), (lda::<ImmediateAddressing>, 2, false), (tax::<ImpliedAddressing>, 2, false), (lax::<ImmediateAddressing>, 2, false),
            (ldy::<AbsoluteAddressing>, 4, false), (lda::<AbsoluteAddressing>, 4, false), (ldx::<AbsoluteAddressing>, 4, false), (lax::<AbsoluteAddressing>, 4, false),

            // 0xb0 - 0xbf
            (bcs::<RelativeAddressing>, 2, true), (lda::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (lax::<IndirectYAddressing>, 5, true),
            (ldy::<ZeroPageXAddressing>, 4, false), (lda::<ZeroPageXAddressing>, 4, false), (ldx::<ZeroPageYAddressing>, 4, false), (lax::<ZeroPageYAddressing>, 4, false),
            (clv::<ImpliedAddressing>, 2, false), (lda::<AbsoluteYAddressing>, 4, true), (tsx::<ImpliedAddressing>, 2, false), (las::<AbsoluteYAddressing>, 4, true),
            (ldy::<AbsoluteXAddressing>, 4, true), (lda::<AbsoluteXAddressing>, 4, true), (ldx::<AbsoluteYAddressing>, 4, true), (lax::<AbsoluteYAddressing>, 4, true),

            // 0xc0 - 0xcf
            (cpy::<ImmediateAddressing>, 2, false), (cmp::<XIndirectAddressing>, 6, false), (nop::<ImmediateAddressing>, 2, false), (dcp::<XIndirectAddressing>, 8, false),
            (cpy::<ZeroPageAddressing>, 3, false), (cmp::<ZeroPageAddressing>, 3, false), (dec::<ZeroPageAddressing>, 5, false), (dcp::<ZeroPageAddressing>, 5, false),
            (iny::<ImpliedAddressing>, 2, false), (cmp::<ImmediateAddressing>, 2, false), (dex::<ImpliedAddressing>, 2, false), (axs::<ImmediateAddressing>, 2, false),
            (cpy::<AbsoluteAddressing>, 4, false), (cmp::<AbsoluteAddressing>, 4, false), (dec::<AbsoluteAddressing>, 6, false), (dcp::<AbsoluteAddressing>, 6, false),

            // 0xd0 - 0xdf
            (bne::<RelativeAddressing>, 2, true), (cmp::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (dcp::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (cmp::<ZeroPageXAddressing>, 4, false), (dec::<ZeroPageXAddressing>, 6, false), (dcp::<ZeroPageXAddressing>, 6, false),
            (cld::<ImpliedAddressing>, 2, false), (cmp::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (dcp::<AbsoluteYAddressing>, 7, true),
            (nop::<AbsoluteXAddressing>, 4, true), (cmp::<AbsoluteXAddressing>, 4, true), (dec::<AbsoluteXAddressing>, 7, false), (dcp::<AbsoluteXAddressing>, 7, false),

            // 0xe0 - 0xef
            (cpx::<ImmediateAddressing>, 2, false), (sbc::<XIndirectAddressing>, 6, false), (nop::<ImmediateAddressing>, 2, false), (isc::<XIndirectAddressing>, 8, false),
            (cpx::<ZeroPageAddressing>, 3, false), (sbc::<ZeroPageAddressing>, 3, false), (inc::<ZeroPageAddressing>, 5, false), (isc::<ZeroPageAddressing>, 5, false),
            (inx::<ImpliedAddressing>, 2, false), (sbc::<ImmediateAddressing>, 2, false), (nop::<ImpliedAddressing>, 2, false), (sbc::<ImmediateAddressing>, 2, false),
            (cpx::<AbsoluteAddressing>, 4, false), (sbc::<AbsoluteAddressing>, 4, false), (inc::<AbsoluteAddressing>, 6, false), (isc::<AbsoluteAddressing>, 6, false),

            // 0xf0 - 0xff
            (beq::<RelativeAddressing>, 2, true), (sbc::<IndirectYAddressing>, 5, true), (kil::<ImpliedAddressing>, 0, false), (isc::<IndirectYAddressing>, 8, false),
            (nop::<ZeroPageXAddressing>, 4, false), (sbc::<ZeroPageXAddressing>, 4, false), (inc::<ZeroPageXAddressing>, 6, false), (isc::<ZeroPageXAddressing>, 6, false),
            (sed::<ImpliedAddressing>, 2, false), (sbc::<AbsoluteYAddressing>, 4, true), (nop::<ImpliedAddressing>, 2, false), (isc::<AbsoluteYAddressing>, 7, false),
            (nop::<AbsoluteXAddressing>, 4, true), (sbc::<AbsoluteXAddressing>, 4, true), (inc::<AbsoluteXAddressing>, 7, false), (isc::<AbsoluteXAddressing>, 7, false),
            ];
}

#[named]
/**
 * ADC implementation converted from c code, taken from https://github.com/DavidBuchanan314/6502-emu/blob/master/6502.c
 */
fn adc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // and read byte
    let b = A::load(c, tgt)?;

    // perform the addition (regs.a+b+C)
    let mut sum: u16;
    if c.is_decimal_set() {
        // bcd
        sum = ((c.regs.a as u16) & 0x0f)
            .wrapping_add((b as u16) & 0x0f)
            .wrapping_add(c.is_carry_set() as u16);
        if sum >= 10 {
            sum = (sum.wrapping_sub(10)) | 10;
            sum = sum
                .wrapping_add((c.regs.a as u16) & 0xf0)
                .wrapping_add((b as u16) & 0xf0);
            if sum > 0x9f {
                sum = sum.wrapping_add(0x60);
            }
        }
    } else {
        // normal
        sum = (c.regs.a as u16)
            .wrapping_add(b as u16)
            .wrapping_add(c.is_carry_set() as u16);
    }

    // set flags
    c.set_carry_flag(sum > 0xff);
    let o = ((c.regs.a as u16) ^ sum) & ((b as u16) ^ sum) & 0x80;
    c.set_overflow_flag(o != 0);
    c.regs.a = (sum & 0xff) as u8;
    c.set_zero_flag(c.regs.a == 0);
    c.set_negative_flag(c.regs.a == 0);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ahx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn alr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn anc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn and<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn arr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn asl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn axs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bcc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bcs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn beq<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bit<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bmi<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bne<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bpl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn brk<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bvc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bvs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn clc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // clear carry
    c.set_carry_flag(false);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cld<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // clear decimal flag
    c.set_decimal_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cli<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // enable interrupts, clear the flag
    c.set_interrupt_disable_flag(false);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn clv<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // clear the overflow flag
    c.set_overflow_flag(false);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cpx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cpy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dcp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dex<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dey<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn eor<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn inc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn isc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn inx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn iny<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn jmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn jsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn kil<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    // panic!
    // TODO: handle debug
    panic!("KIL\n--> {}", c.regs);
}

#[named]
fn las<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lda<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // load byte to A
    let b = A::load(c, tgt)?;
    c.regs.a = b;

    // set flags
    c.set_negative_flag(c.regs.a > 0x7f);
    c.set_zero_flag(c.regs.a == 0);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ldx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;
    // load byte to X
    let b = A::load(c, tgt)?;
    c.regs.x = b;

    // set flags
    c.set_negative_flag(c.regs.x > 0x7f);
    c.set_zero_flag(c.regs.x == 0);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ldy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn nop<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ora<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn pha<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn php<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn pla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn plp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rol<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ror<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rra<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rti<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rts<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
/**
 * SBC implementation converted from c code, taken from https://github.com/DavidBuchanan314/6502-emu/blob/master/6502.c
 */
fn sbc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // and read byte
    let b = A::load(c, tgt)?;

    // perform non-bcd subtraction (regs.a-b-1+C)
    let sub: u16 = (c.regs.a as u16)
        .wrapping_sub(b as u16)
        .wrapping_sub(1)
        .wrapping_add(c.is_carry_set() as u16);

    if c.is_decimal_set() {
        // bcd
        let mut lo: u8 = (c.regs.a & 0x0f)
            .wrapping_sub(b & 0x0f)
            .wrapping_sub(1)
            .wrapping_add(c.is_carry_set());
        let mut hi: u8 = (c.regs.a >> 4).wrapping_sub(b >> 4);
        if lo & 0x10 != 0 {
            lo = lo.wrapping_sub(6);
            hi = hi.wrapping_sub(1);
        }
        if hi & 0x10 != 0 {
            hi = hi.wrapping_sub(6);
        }
        c.regs.a = (hi << 4) | (lo & 0x0f);
    } else {
        // normal
        c.regs.a = (sub & 0xff) as u8;
    }

    // set flags
    c.set_carry_flag(sub < 0x100);
    let o = ((c.regs.a as u16) ^ sub) & ((b as u16) ^ sub) & 0x80;
    c.set_overflow_flag(o != 0);
    c.set_zero_flag(c.regs.a == 0);
    c.set_negative_flag(c.regs.a == 0);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // set carry
    c.set_carry_flag(true);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sed<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // set decimal flag
    c.set_decimal_flag(true);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sei<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    // disable interrupts
    c.set_interrupt_disable_flag(true);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn shx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn shy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn slo<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sre<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sta<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn stx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sty<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tas<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tay<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tsx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn txa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn txs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tya<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn xaa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(u16, usize), MemoryError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    c.debug_out_opcode::<A>(function_name!())?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}
