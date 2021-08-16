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

use crate::cpu::addressing_modes::AddressingModeId::*;
use crate::cpu::addressing_modes::RelativeAddressing;
use crate::cpu::addressing_modes::*;
use crate::cpu::cpu_error;
use crate::cpu::cpu_error::CpuError;
use crate::cpu::Cpu;
use crate::utils;
use crate::utils::*;
use ::function_name::named;
use lazy_static::*;

/**
 * holds opcode information for assembler/diassembler
 */
#[derive(Clone, Debug, Copy)]
pub(crate) struct OpcodeMarker {
    pub(crate) name: &'static str,
    pub(crate) id: AddressingModeId,
}

lazy_static! {
    /**
     * the 256 opcodes table (includes undocumented)
     *
     * each opcode gets in input a reference to the Cpu, the cycles needed to execute the opcode, a boolean to indicate if, on crossing page boundaries, an extra cycles must be added and
     * another boolean to indicate decoding only (no execution, for the disassembler).
     * returns a tuple with the instruction size and the effective elapsed cycles (may include the aferomentioned additional cycle).
     *
     * for clarity, each Vec element is a tuple defined as this (with named return values):
     *
     * (< fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only: bool) -> Result<(instr_size:i8, out_cycles:usize), CpuError>, in_cycles: usize, add_extra_cycle:bool) >)
     *
     */
    pub(crate) static ref OPCODE_MATRIX: Vec<( fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only:bool) -> Result<(i8, usize), CpuError>, usize, bool, OpcodeMarker)> =
        vec![
            // 0x0 - 0xf
            (brk::<ImpliedAddressing>, 7, false, OpcodeMarker{ name: "brk", id: Imp}), (ora::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "ora", id: Xin}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (slo::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "slo", id: Xin}),
            (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}), (ora::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ora", id: Zpg}), (asl::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "asl", id: Zpg}), (slo::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "slo", id: Zpg}),
            (php::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "php", id: Imp}), (ora::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ora", id: Imm}), (asl::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "asl", id: Acc}), (anc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "anc", id: Imm}),
            (nop::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Abs}), (ora::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Abs}), (asl::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Abs}), (slo::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "slo", id: Abs}),

            // 0x10 - 0x1f
            (bpl::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bpl", id: Rel}), (ora::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "ora", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (slo::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "slo", id: Iny}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (ora::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Zpx}), (asl::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Zpx}), (slo::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "slo", id: Zpx}),
            (clc::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clc", id: Imp}), (ora::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Abx}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (slo::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "slo", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (ora::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Abx}), (asl::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "asl", id: Abx}), (slo::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "slo", id: Abx}),

            // 0x20 - 0x2f
            (jsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "jsr", id: Abs}), (and::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "and", id: Abx}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (rla::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "rla", id: Xin}),
            (bit::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "bit", id: Zpg}), (and::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "and", id: Zpg}), (rol::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rol", id: Zpg}), (rla::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rla", id: Zpg}),
            (plp::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "plp", id: Imp}), (and::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "and", id: Imm}), (rol::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "rol", id: Acc}), (anc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "anc", id: Imm}),
            (bit::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "bit", id: Abs}), (and::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "and", id: Abs}), (rol::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Abs}), (rla::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rla", id: Abs}),

            // 0x30 - 0x3f
            (bmi::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bmi", id: Rel}), (and::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "and", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (rla::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "rla", id: Iny}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (and::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "and", id: Zpx}), (rol::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Zpx}), (rla::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rla", id: Zpx}),
            (sec::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sec", id: Imp}), (and::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "and", id: Aby}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (rla::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "rla", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (and::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "and", id: Abx}), (rol::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rol", id: Abx}), (rla::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rla", id: Abx}),

            // 0x40 - 0x4f
            (rti::<ImpliedAddressing>, 6, false, OpcodeMarker{ name: "rti", id: Imp}), (eor::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "eor", id: Xin}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (sre::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "sre", id: Xin}),
            (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}), (eor::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "eor", id: Zpg}), (lsr::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "lsr", id: Zpg}), (sre::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "sre", id: Zpg}),
            (pha::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "pha", id: Imp}), (eor::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "eor", id: Imm}), (lsr::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "lsr", id: Acc}), (alr::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "alr", id: Imm}),
            (jmp::<AbsoluteAddressing>, 3, false, OpcodeMarker{ name: "jmp", id: Abs}), (eor::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Abs}), (lsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Abs}), (sre::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "sre", id: Abs}),

            // 0x50 - 0x5f
            (bvc::<ImpliedAddressing>, 2, true, OpcodeMarker{ name: "bvc", id: Imp}), (eor::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "eor", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (sre::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "sre", id: Iny}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (eor::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Zpx}), (lsr::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Zpx}), (sre::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "sre", id: Zpx}),
            (cli::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cli", id: Imp}), (eor::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Aby}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (sre::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "sre", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (eor::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Abx}), (lsr::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "lsr", id: Abx}), (sre::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "sre", id: Abx}),

            // 0x60 - 0x6f
            (rts::<RelativeAddressing>, 6, false, OpcodeMarker{ name: "rts", id: Rel}), (adc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "adc", id: Xin}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (rra::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "rra", id: Xin}),
            (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}), (adc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "adc", id: Zpg}), (ror::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "ror", id: Zpg}), (rra::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rra", id: Zpg}),
            (pla::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "pla", id: Imp}), (adc::<ImmediateAddressing>, 2, true, OpcodeMarker{ name: "adc", id: Imm}), (ror::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "ror", id: Acc}), (arr::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "arr", id: Imm}),
            (jmp::<IndirectAddressing>, 5, false, OpcodeMarker{ name: "jmp", id: Ind}), (adc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Abs}), (ror::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Abs}), (rra::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rra", id: Abs}),

            // 0x70 - 0x7f
            (bvs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bvs", id: Rel}), (adc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "adc", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (rra::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "rra", id: Iny}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (adc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Zpx}), (ror::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Zpx}), (rra::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rra", id: Zpx}),
            (sei::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sei", id: Imp}), (adc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Aby}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (rra::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "rra", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (adc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Abx}), (ror::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "ror", id: Abx}), (rra::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rra", id: Abx}),

            // 0x80 - 0x8f
            (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (sta::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Xin}), (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (sax::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sax", id: Xin}),
            (sty::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sty", id: Zpg}), (sta::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sta", id: Zpg}), (stx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "stx", id: Zpg}), (sax::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sax", id: Zpg}),
            (dey::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dey", id: Imp}), (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (txa::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txa", id: Imp}), (xaa::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "xaa", id: Imm}),
            (sty::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Abs}), (sta::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Abs}), (stx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Abs}), (sax::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sax", id: Abs}),

            // 0x90 - 0x9f
            (bcc::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcc", id: Rel}), (sta::<IndirectYAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (ahx::<IndirectYAddressing>, 6, false, OpcodeMarker{ name: "ahx", id: Iny}),
            (sty::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Zpx}), (sta::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Zpx}), (stx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Zpy}), (sax::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "sax", id: Zpy}),
            (tya::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tya", id: Imp}), (sta::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Aby}), (txs::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txs", id: Imp}), (tas::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "tas", id: Aby}),
            (shy::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "shy", id: Abx}), (sta::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Abx}), (shx::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "shx", id: Aby}), (ahx::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "ahx", id: Aby}),

            // 0xa0 - 0xaf
            (ldy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldy", id: Imm}), (lda::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "lda", id: Xin}), (ldx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldx", id: Imm}), (lax::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "lax", id: Xin}),
            (ldy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldy", id: Zpg}), (lda::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "lda", id: Zpg}), (ldx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldx", id: Zpg}), (lax::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "lax", id: Zpg}),
            (tay::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tay", id: Imp}), (lda::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lda", id: Imm}), (tax::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tax", id: Imp}), (lax::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lax", id: Imm}),
            (ldy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Abs}), (lda::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Abs}), (ldx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Abs}), (lax::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Abs}),

            // 0xb0 - 0xbf
            (bcs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcs", id: Rel}), (lda::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lda", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (lax::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lax", id: Iny}),
            (ldy::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Zpx}), (lda::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Zpx}), (ldx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Zpy}), (lax::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Zpy}),
            (clv::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clv", id: Imp}), (lda::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Aby}), (tsx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tsx", id: Imp}), (las::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "las", id: Aby}),
            (ldy::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ldy", id: Abx}), (lda::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Abx}), (ldx::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ldx", id: Aby}), (lax::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lax", id: Aby}),

            // 0xc0 - 0xcf
            (cpy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpy", id: Imm}), (cmp::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "cmp", id: Xin}), (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (dcp::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "dcp", id: Xin}),
            (cpy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpy", id: Zpg}), (cmp::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cmp", id: Zpg}), (dec::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dec", id: Zpg}), (dcp::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dcp", id: Zpg}),
            (iny::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "iny", id: Imp}), (cmp::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cmp", id: Imm}), (dex::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dex", id: Imp}), (axs::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "axs", id: Imm}),
            (cpy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpy", id: Abs}), (cmp::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Abs}), (dec::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Abs}), (dcp::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "dcp", id: Abs}),

            // 0xd0 - 0xdf
            (bne::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bne", id: Rel}), (cmp::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "cmp", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (dcp::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "dcp", id: Iny}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (cmp::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Zpx}), (dec::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Zpx}), (dcp::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "dcp", id: Zpx}),
            (cld::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cld", id: Imp}), (cmp::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Aby}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (dcp::<AbsoluteYAddressing>, 7, true, OpcodeMarker{ name: "dcp", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (cmp::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Abx}), (dec::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "dec", id: Abx}), (dcp::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "dcp", id: Abx}),

            // 0xe0 - 0xef
            (cpx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpx", id: Imm}), (sbc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sbc", id: Xin}), (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (isc::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "isc", id: Xin}),
            (cpx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpx", id: Zpg}), (sbc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sbc", id: Zpg}), (inc::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "inc", id: Zpg}), (isc::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "isc", id: Zpg}),
            (inx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "inx", id: Imp}), (sbc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbc", id: Imm}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (sbc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbc", id: Imm}),
            (cpx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpx", id: Abs}), (sbc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Abs}), (inc::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Abs}), (isc::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "isc", id: Abs}),

            // 0xf0 - 0xff
            (beq::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "beq", id: Rel}), (sbc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "sbc", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (isc::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "isc", id: Ind}),
            (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}), (sbc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Zpx}), (inc::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Zpx}), (isc::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "isc", id: Zpx}),
            (sed::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sed", id: Imp}), (sbc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Aby}), (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}), (isc::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "isc", id: Aby}),
            (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}), (sbc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Abx}), (inc::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "inc", id: Abx}), (isc::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "isc", id: Abx}),
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
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((0, 0));
    }

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
    c.set_negative_flag(utils::is_signed(c.regs.a));
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ahx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn alr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn anc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn and<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }
    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn arr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn asl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn axs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bcc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bcs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn beq<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bit<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bmi<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bne<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bpl<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn brk<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bvc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn bvs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn clc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear carry
    c.set_carry_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cld<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear decimal flag
    c.set_decimal_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cli<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // enable interrupts, clear the flag
    c.set_interrupt_disable_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn clv<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear the overflow flag
    c.set_overflow_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cpx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn cpy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dcp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dex<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn dey<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn eor<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn inc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn isc<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn inx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn iny<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn jmp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn jsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn kil<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    // this is an invalid opcode and emulation should be halted!
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    Err(cpu_error::new_invalid_opcode_error(c.regs.pc as usize))
}

#[named]
fn las<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lda<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // load byte to A
    let b = A::load(c, tgt)?;
    c.regs.a = b;

    // set flags
    c.set_negative_flag(utils::is_signed(c.regs.a));
    c.set_zero_flag(c.regs.a == 0);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ldx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // load byte to X
    let b = A::load(c, tgt)?;
    c.regs.x = b;

    // set flags
    c.set_negative_flag(utils::is_signed(c.regs.a));
    c.set_zero_flag(c.regs.x == 0);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ldy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // load byte to Y
    let b = A::load(c, tgt)?;
    c.regs.y = b;

    // set flags
    c.set_negative_flag(utils::is_signed(c.regs.a));
    c.set_zero_flag(c.regs.x == 0);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn lsr<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn nop<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // noop, do nothing ...
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ora<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn pha<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn php<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn pla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn plp<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rla<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rol<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn ror<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rra<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rti<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn rts<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
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
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

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
    c.set_negative_flag(utils::is_signed(c.regs.a));
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // set carry
    c.set_carry_flag(true);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sed<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // set decimal flag
    c.set_decimal_flag(true);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sei<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // disable interrupts
    c.set_interrupt_disable_flag(true);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn shx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn shy<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn slo<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sre<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sta<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // store A in memory
    A::store(c, tgt, c.regs.a)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn stx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sty<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tas<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tay<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tsx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn txa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn txs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // X -> S
    c.regs.s = c.regs.x;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn tya<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn xaa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    debug_out_opcode::<A>(c, function_name!())?;
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}
