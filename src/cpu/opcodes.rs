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

use crate::cpu::addressing_modes;
use crate::cpu::addressing_modes::AddressingModeId::*;
use crate::cpu::addressing_modes::*;
use crate::cpu::cpu_error;
use crate::cpu::cpu_error::CpuError;
use crate::cpu::debugger::Debugger;
use crate::cpu::{Cpu, Vectors};
use crate::utils;
use crate::utils::*;
use ::function_name::named;
use lazy_static::*;

/**
 * holds opcode information for assembler/disassembler
 */
#[derive(Clone, Debug, Copy)]
pub(crate) struct OpcodeMarker {
    /// opcode name
    pub(crate) name: &'static str,

    /// addressing mode
    pub(crate) id: AddressingModeId,
}

lazy_static! {
    /**
     * the 256 opcodes table (includes undocumented)
     *
     * each opcode gets in input a reference to the Cpu, a reference to the Debugger, the cycles needed to execute the opcode, a boolean to indicate if, on crossing page boundaries, an extra cycles must be added,
     * a boolean to indicate decoding only (no execution, for the disassembler), a boolean to indicate if an rw breakpoint has been triggered before and finally a boolean to silence outputs for combined opcodes (i.e ISC).
     * returns a tuple with the instruction size and the effective elapsed cycles (may include the aferomentioned additional cycle).
     *
     * for clarity, each Vec element is a tuple defined as this (with named return values):
     *
     * (< fn(c: &mut Cpu, d: &Debugger, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only: bool, rw_bp_triggered: bool, quiet: bool) -> Result<(instr_size:i8, out_cycles:usize), CpuError>, d: &Debugger, in_cycles: usize, add_extra_cycle:bool) >)
     *
     * most of the info taken from :
     *
     * - https://www.masswerk.at/6502/6502_instruction_set.html#SHA
     * - https://problemkaputt.de/2k6specs.htm#cpu65xxmicroprocessor
     * - http://www.oxyron.de/html/opcodes02.html
     * - http://www.obelisk.me.uk/6502/reference.html
     */
    pub(crate) static ref OPCODE_MATRIX: Vec<( fn(c: &mut Cpu, d: &Debugger, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only:bool, rw_bp_triggered: bool, quiet: bool) -> Result<(i8, usize), CpuError>, usize, bool, OpcodeMarker)> =
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
            (tay::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tay", id: Imp}), (lda::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lda", id: Imm}), (tax::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tax", id: Imp}), (lax::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lxa", id: Imm}),
            (ldy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Abs}), (lda::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Abs}), (ldx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Abs}), (lax::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Abs}),

            // 0xb0 - 0xbf
            (bcs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcs", id: Rel}), (lda::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lda", id: Iny}), (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}), (lax::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lax", id: Iny}),
            (ldy::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Zpx}), (lda::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Zpx}), (ldx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Zpy}), (lax::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Zpy}),
            (clv::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clv", id: Imp}), (lda::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Aby}), (tsx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tsx", id: Imp}), (las::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "las", id: Aby}),
            (ldy::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ldy", id: Abx}), (lda::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Abx}), (ldx::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ldx", id: Aby}), (lax::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lax", id: Aby}),

            // 0xc0 - 0xcf
            (cpy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpy", id: Imm}), (cmp::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "cmp", id: Xin}), (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}), (dcp::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "dcp", id: Xin}),
            (cpy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpy", id: Zpg}), (cmp::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cmp", id: Zpg}), (dec::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dec", id: Zpg}), (dcp::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dcp", id: Zpg}),
            (iny::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "iny", id: Imp}), (cmp::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cmp", id: Imm}), (dex::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dex", id: Imp}), (sbx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbx", id: Imm}),
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

/**
 * helper to set Z and N flags in one shot, depending on val
 */
fn set_zn_flags(c: &mut Cpu, val: u8) {
    c.set_zero_flag(val == 0);
    c.set_negative_flag(utils::is_signed(val));
}

/**
 * push word on the stack
 */
fn push_word_le(c: &mut Cpu, w: u16) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    mem.write_word_le(0x100 + c.regs.s as usize, w)?;
    c.regs.s = c.regs.s.wrapping_sub(2);
    Ok(())
}

/**
 * push word on the stack
 */
fn push_byte(c: &mut Cpu, b: u8) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    mem.write_byte(0x100 + c.regs.s as usize, b)?;
    c.regs.s = c.regs.s.wrapping_sub(1);
    Ok(())
}

#[named]
/**
 * ADC - Add with Carry
 *
 * A,Z,C,N = A+M+C
 *
 * This instruction adds the contents of a memory location to the accumulator together with the carry bit.
 * If overflow occurs the carry bit is set, this enables multiple byte addition to be performed.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set if overflow in bit 7
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Set if sign bit is incorrect
 * N	Negative Flag	Set if bit 7 set
 *
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	ADC #oper	69	2	    2  
 * zeropage	ADC oper	65	2	3  
 * zeropage,X	ADC oper,X	75	2	    4  
 * absolute	ADC oper	6D	3	4  
 * absolute,X	ADC oper,X	7D	3	    4*
 * absolute,Y	ADC oper,Y	79	3	    4*
 * (indirect,X)	ADC (oper,X)	61	2	6  
 * (indirect),Y	ADC (oper),Y	71	2	5*
 *
 * ADC implementation (including decimal mode support) converted from c code, taken from https://github.com/DavidBuchanan314/6502-emu/blob/master/6502.c
 */
fn adc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((0, 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

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
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * AHX (undoc) (aka SHA, aka AXA)
 *
 * Stores A AND X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is dropped, page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
 *
 * A AND X AND (H+1) -> M
 *
 * N	Z	C	I	D	V
 * -	-	-	-	-	-
 *
 * addressing	assembler	    opc	bytes	cycles
 * absolut,Y	SHA oper,Y	    9F	3	    5  	†
 * (indirect),Y	SHA (oper),Y	93	2	    6  	†
 */
#[named]
fn ahx<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // get msb from target address
    let h = (tgt >> 8) as u8;

    // A & X & (H + 1)
    let res = c.regs.a & c.regs.x & h.wrapping_add(1);

    // store
    A::store(c, d, tgt, rw_bp_triggered, res)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ALR (undoc) (aka ASR)
 *
 * AND oper + LSR
 *
 * A AND oper, 0 -> [76543210] -> C
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	ALR #oper	4B	2	    2
 */
#[named]
fn alr<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // set carry flag with bit to be shifted out
    c.set_carry_flag(c.regs.a & 1 != 0);

    // and
    c.regs.a = c.regs.a & b;
    // lsr
    c.regs.a >>= 1;

    set_zn_flags(c, c.regs.a);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ANC (undoc)
 *
 * AND oper + set C as ASL
 *
 * A AND oper, bit(7) -> C
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	ANC #oper	0B	2	    2
 */
#[named]
fn anc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // and
    c.regs.a = c.regs.a & b;
    set_zn_flags(c, c.regs.a);

    // note to ANC: this command performs an AND operation only, but bit 7 is put into the carry, as if the ASL/ROL would have been executed.
    c.set_carry_flag(utils::is_signed(c.regs.a));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
* AND - Logical AND
*
* A,Z,N = A&M
*
* A logical AND is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
*
* Processor Status after use:
*
* C	Carry Flag	Not affected
* Z	Zero Flag	Set if A = 0
* I	Interrupt Disable	Not affected
* D	Decimal Mode Flag	Not affected
* B	Break Command	Not affected
* V	Overflow Flag	Not affected
* N    Negative Flag	Set if bit 7 set
*
* addressing	assembler	    opc	bytes	cycles
* immediate	AND #oper	    29	2	    2
* zeropage	AND oper	        25	2	    3
* zeropage,X	AND oper,X	    35	2	    4
* absolute	AND oper	        2D	3	    4
* absolute,X	AND oper,X	    3D	3	    4*
* absolute,Y	AND oper,Y	    39	3	    4*
* (indirect,X)	AND (oper,X)	21	2	    6
* (indirect),Y	AND (oper),Y	31	2	    5*

*/
#[named]
fn and<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // A AND M -> A
    c.regs.a = c.regs.a & b;

    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ARR (undoc)
 *
 * AND oper + ROR
 *
 * This operation involves the adder:
 *
 * V-flag is set according to (A AND oper) + oper
 * The carry is not set, but bit 7 (sign) is exchanged with the carry
 *
 * A AND oper, C -> [76543210] -> C
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	+
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	ARR #oper	6B	2	    2
 */
#[named]
fn arr<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // perform and
    c.regs.a = c.regs.a & b;

    // handle overflow flag
    c.set_overflow_flag((c.regs.a as usize + b as usize) > 0xff);

    // save carry and bit 7 of A
    let previous_c: bool = c.is_carry_set();
    let previous_bit7 = utils::is_signed(c.regs.a);

    // ror
    c.regs.a = c.regs.a >> 1;

    // swap carry and bit 7 of A
    c.set_carry_flag(previous_bit7);
    if previous_c {
        c.regs.a |= 0x80;
    }
    set_zn_flags(c, c.regs.a);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ASL - Arithmetic Shift Left
 *
 * A,Z,C,N = M*2 or M,Z,C,N = M*2
 *
 * This operation shifts all the bits of the accumulator or memory contents one bit left. Bit 0 is set to 0 and bit 7 is placed in the carry flag.
 * The effect of this operation is to multiply the memory contents by 2 (ignoring 2's complement considerations), setting the carry if the result will not fit in 8 bits.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set to contents of old bit 7
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * accumulator	ASL A	    0A	1	    2  
 * zeropage	ASL oper	    06	2	    5  
 * zeropage,X	ASL oper,X	16	2	    6  
 * absolute	ASL oper	    0E	3	    6  
 * absolute,X	ASL oper,X	1E	3	    7
 */
#[named]
fn asl<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let mut b = A::load(c, d, tgt, rw_bp_triggered)?;
    c.set_carry_flag(utils::is_signed(b));

    // shl
    b <<= 1;
    c.set_zero_flag(c.regs.a == 0);
    c.set_negative_flag(utils::is_signed(b));

    // store back
    A::store(c, d, tgt, rw_bp_triggered, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * BCC - Branch if Carry Clear
 *
 * If the carry flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BCC oper	90	2	    2**
 */
#[named]
fn bcc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if !c.is_carry_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BCS - Branch if Carry Set
 *
 * If the carry flag is set then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BCS oper	B0	2	    2**
 */

#[named]
fn bcs<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if c.is_carry_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BEQ - Branch if Equal
 *
 * If the zero flag is set then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BEQ oper	F0	2	    2**
 */
#[named]
fn beq<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if c.is_zero_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
* BIT - Bit Test
*
* A & M, N = M7, V = M6
*
* This instructions is used to test if one or more bits are set in a target memory location.
* The mask pattern in A is ANDed with the value in memory to set or clear the zero flag, but the result is not kept.
*
* Bits 7 and 6 of the value from memory are copied into the N and V flags.
*
* Processor Status after use:
*
* C	Carry Flag	Not affected
* Z	Zero Flag	Set if the result of the AND is zero
* I	Interrupt Disable	Not affected
* D	Decimal Mode Flag	Not affected
* B	Break Command	Not affected
* V	Overflow Flag	Set to bit 6 of the memory value
* N	Negative Flag	Set to bit 7 of the memory value
*
* addressing	assembler	opc	bytes	cycles
* zeropage	BIT oper	    24	2	    3
* absolute	BIT oper	    2C	3	    4

*/
#[named]
fn bit<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    let and_res = c.regs.a & b;

    c.set_zero_flag(and_res == 0);
    c.set_negative_flag(utils::is_signed(b));
    c.set_overflow_flag(b & 0b01000000 != 0);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * BMI - Branch if Minus
 *
 * If the negative flag is set then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BMI oper	30	2	    2**
 */
#[named]
fn bmi<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if c.is_negative_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BNE - Branch if Not Equal
 *
 * If the zero flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BNE oper	D0	2	    2**
 */
#[named]
fn bne<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if !c.is_zero_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BPL - Branch if Positive
 *
 * If the negative flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BPL oper	10	2	    2**
 */
#[named]
fn bpl<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if !c.is_negative_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BRK - Force Interrupt
 *
 * The BRK instruction forces the generation of an interrupt request.
 * The program counter and processor status are pushed on the stack then the IRQ interrupt vector at $FFFE/F is loaded into the PC and the break flag in the status set to one.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Set to 1
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    BRK	        00	1	    7  
 */
#[named]
fn brk<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // push pc and p on stack
    push_word_le(c, c.regs.pc)?;
    push_byte(c, c.regs.p)?;

    // set break flag
    c.set_break_flag(true);

    // set pc to irq
    c.regs.pc = Vectors::IRQ as u16;
    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * BVC - Branch if Overflow Clear
 *
 * If the overflow flag is clear then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BVC oper	50	2	    2**
 */
#[named]
fn bvc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if !c.is_overflow_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * BVS - Branch if Overflow Set
 *
 * If the overflow flag is set then add the relative displacement to the program counter to cause a branch to a new location.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * relative	    BVS oper	70	2	    2**
 */
#[named]
fn bvs<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if c.is_overflow_set() {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b as u16);
        c.regs.pc = new_pc;
    }

    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
    ))
}

/**
 * CLC - Clear Carry Flag
 *
 * C = 0
 *
 * Set the carry flag to zero.
 *
 * C	Carry Flag	Set to 0
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    CLC	        18	1	    2  
 */
#[named]
fn clc<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear carry
    c.set_carry_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CLD - Clear Decimal Mode
 *
 * D = 0
 *
 * Sets the decimal mode flag to zero.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Set to 0
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    CLD	        D8	1	    2  
 */
#[named]
fn cld<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear decimal flag
    c.set_decimal_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CLI - Clear Interrupt Disable
 *
 * I = 0
 *
 * Clears the interrupt disable flag allowing normal interrupt requests to be serviced.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Set to 0
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    CLI	        58	1	    2  
 */
#[named]
fn cli<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // enable interrupts, clear the flag
    c.set_interrupt_disable_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CLV - Clear Overflow Flag
 *
 * V = 0
 *
 * Clears the overflow flag.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Set to 0
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    CLV	        B8	1	    2  
 */
#[named]
fn clv<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // clear the overflow flag
    c.set_overflow_flag(false);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CMP - Compare
 *
 * Z,C,N = A-M
 *
 * This instruction compares the contents of the accumulator with another memory held value and sets the zero and carry flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set if A >= M
 * Z	Zero Flag	Set if A = M
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	    opc	bytes	cycles
 * immediate	CMP #oper	    C9	2	    2  
 * zeropage	    CMP oper	    C5	2	    3  
 * zeropage,X	CMP oper,X	    D5	2	    4  
 * absolute	    CMP oper	    CD	3	    4  
 * absolute,X	CMP oper,X	    DD	3	    4*
 * absolute,Y	CMP oper,Y	    D9	3	    4*
 * (indirect,X)	CMP (oper,X)	C1	2	    6  
 * (indirect),Y	CMP (oper),Y	D1	2	    5*
 */
#[named]
fn cmp<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    let res = c.regs.a.wrapping_sub(b);
    c.set_carry_flag(c.regs.a >= b);
    c.set_zero_flag(c.regs.a == b);
    c.set_negative_flag(utils::is_signed(res));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CPX - Compare X Register
 *
 * Z,C,N = X-M
 *
 * This instruction compares the contents of the X register with another memory held value and sets the zero and carry flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set if X >= M
 * Z	Zero Flag	Set if X = M
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	CPX #oper	E0	2	    2  
 * zeropage	    CPX oper	E4	2	    3  
 * absolute	    CPX oper	EC	3	    4  
 */
#[named]
fn cpx<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    let res = c.regs.x.wrapping_sub(b);
    c.set_carry_flag(c.regs.x >= b);
    c.set_zero_flag(c.regs.x == b);
    c.set_negative_flag(utils::is_signed(res));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * CPY - Compare Y Register
 *
 * Z,C,N = Y-M
 *
 * This instruction compares the contents of the Y register with another memory held value and sets the zero and carry flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set if Y >= M
 * Z	Zero Flag	Set if Y = M
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	CPY #oper	C0	2	    2  
 * zeropage	    CPY oper	C4	2	    3  
 * absolute	    CPY oper	CC	3	    4  
 */
#[named]
fn cpy<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    let res = c.regs.y.wrapping_sub(b);
    c.set_carry_flag(c.regs.y >= b);
    c.set_zero_flag(c.regs.y == b);
    c.set_negative_flag(utils::is_signed(res));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * DCP (undoc) (aka DCM)
 *
 * DEC oper + CMP oper
 *
 * M - 1 -> M, A - M
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	    opc	bytes	cycles
 * zeropage	    DCP oper	    C7	2	    5
 * zeropage,X	DCP oper,X	    D7	2	    6
 * absolute	    DCP oper	    CF	3	    6
 * absolut,X	DCP oper,X	    DF	3	    7
 * absolut,Y	DCP oper,Y	    DB	3	    7
 * (indirect,X)	DCP (oper,X)	C3	2	    8
 * (indirect),Y	DCP (oper),Y	D3	2	    8
 */
#[named]
fn dcp<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // C0+yy        nzc---  DCP op   DEC+CMP   op=op-1     // A-op
    // dec
    let mut b = A::load(c, d, tgt, rw_bp_triggered)?;
    b = b.wrapping_sub(1);
    A::store(c, d, tgt, false, b)?;

    // cmp
    let res = c.regs.a.wrapping_sub(b);
    c.set_carry_flag(c.regs.a >= b);
    c.set_zero_flag(c.regs.a == b);
    c.set_negative_flag(utils::is_signed(res));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
* DEC - Decrement Memory
*
* M,Z,N = M-1
*
* Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
*
* Processor Status after use:
*
* C	Carry Flag	Not affected
* Z	Zero Flag	Set if result is zero
* I	Interrupt Disable	Not affected
* D	Decimal Mode Flag	Not affected
* B	Break Command	Not affected
* V	Overflow Flag	Not affected
* N	Negative Flag	Set if bit 7 of the result is set
*
* addressing	assembler	opc	bytes	cycles
* zeropage	    DEC oper	C6	2	    5  
* zeropage,X	DEC oper,X	D6	2	    6  
* absolute	    DEC oper	CE	3	    3  
* absolute,X	DEC oper,X	DE	3	    7  
*/
#[named]
fn dec<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let mut b = A::load(c, d, tgt, rw_bp_triggered)?;
    b = b.wrapping_sub(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, false, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * DEX - Decrement X Register
 *
 * X,Z,N = X-1
 *
 * Subtracts one from the X register setting the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if X is zero
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of X is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    DEX	        CA	1	    2  
*/
#[named]
fn dex<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    c.regs.x = c.regs.x.wrapping_sub(1);
    set_zn_flags(c, c.regs.x);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * DEY - Decrement Y Register
 *
 * Y,Z,N = Y-1
 *
 * Subtracts one from the Y register setting the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if Y is zero
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of Y is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    DEY	        88	1	    2  
 */
#[named]
fn dey<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    c.regs.y = c.regs.y.wrapping_sub(1);
    set_zn_flags(c, c.regs.y);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * EOR - Exclusive OR
 *
 * A,Z,N = A^M
 *
 * An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 set
 *
 * addressing	assembler	    opc	bytes	cycles
 * immediate	EOR #oper	    49	2	    2  
 * zeropage	    EOR oper	    45	2	    3  
 * zeropage,X	EOR oper,X	    55	2	    4  
 * absolute	    EOR oper	    4D	3	    4  
 * absolute,X	EOR oper,X	    5D	3	    4*
 * absolute,Y	EOR oper,Y	    59	3	    4*
 * (indirect,X)	EOR (oper,X)	41	2	    6  
 * (indirect),Y	EOR (oper),Y	51	2	    5*
 */

#[named]
fn eor<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    c.regs.a = c.regs.a ^ b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * INC - Increment Memory
 *
 * M,Z,N = M+1
 *
 * Adds one to the value held at a specified memory location setting the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if result is zero
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * zeropage	    INC oper	E6	2	    5  
 * zeropage,X	INC oper,X	F6	2	    6  
 * absolute	    INC oper	EE	3	    6  
 * absolute,X	INC oper,X	FE	3	    7
*/
#[named]
fn inc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let mut b = A::load(c, d, tgt, rw_bp_triggered)?;

    b = b.wrapping_add(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, rw_bp_triggered, b)?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * INX - Increment X Register
 *
 * X,Z,N = X+1
 *
 * Adds one to the X register setting the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if X is zero
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of X is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    INX	        E8	1	    2  
 */
#[named]
fn inx<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    c.regs.x = c.regs.x.wrapping_add(1);
    set_zn_flags(c, c.regs.x);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * INY - Increment Y Register
 *
 * Y,Z,N = Y+1
 *
 * Adds one to the Y register setting the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if Y is zero
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of Y is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    INY	        C8	1	    2  
 */
#[named]
fn iny<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    c.regs.y = c.regs.x.wrapping_add(1);
    set_zn_flags(c, c.regs.y);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ISC (undoc) (aka ISB, aka INS)
 *
 * INC oper + SBC oper
 *
 * M + 1 -> M, A - M - C -> A
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	+
 *
 * addressing	assembler	    opc	bytes	cycles
 * zeropage	    ISC oper	    E7	2	    5
 * zeropage,X	ISC oper,X	    F7	2	    6
 * absolute	    ISC oper	    EF	3	    6
 * absolut,X	ISC oper,X	    FF	3	    7
 * absolut,Y	ISC oper,Y	    FB	3	    7
 * (indirect,X)	ISC (oper,X)	E3	2	    8
 * (indirect),Y	ISC (oper),Y	F3	2	    4
 */

#[named]
fn isc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // perform inc + sbc internally
    inc::<A>(c, d, 0, false, decode_only, rw_bp_triggered, true)?;
    sbc::<A>(c, d, 0, false, decode_only, rw_bp_triggered, true)?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * JMP - Jump
 *
 * Sets the program counter to the address specified by the operand.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * absolute	    JMP oper	4C	3	    3  
 * indirect	    JMP (oper)	6C	3	    5  
 */
#[named]
fn jmp<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // set pc
    c.regs.pc = tgt;

    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * JSR - Jump to Subroutine
 *
 * The JSR instruction pushes the address (minus one) of the return point on to the stack and then sets the program counter to the target memory address.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * absolute	    JSR oper	20	3	    6  
 */
#[named]
fn jsr<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // push return address
    push_word_le(c, c.regs.pc.wrapping_add(A::len() as u16).wrapping_sub(1))?;

    // set pc
    c.regs.pc = tgt;

    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn kil<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    _: usize,
    _: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    // this is an invalid opcode and emulation should be halted!
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // invalid !
    Err(cpu_error::new_invalid_opcode_error(c.regs.pc as usize))
}

/**
 * LAS (undoc) (aka LAR)
 *
 * LDA/TSX oper
 *
 * M AND SP -> A, X, SP
 *
 * N	Z	C	I	D	V
 * +	+	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * absolut,Y	LAS oper,Y	BB	3	    4*
 */

#[named]
fn las<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // get operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;
    let res = b & c.regs.s;

    c.regs.a = res;
    c.regs.x = res;
    c.regs.s = res;
    set_zn_flags(c, res);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LAX
 *
 * LDA oper + LDX oper
 *
 * M -> A -> X
 *
 * N	Z	C	I	D	V
 * +	+	-	-	-	-
 *
 * addressing	    assembler	    opc	bytes	cycles
 * zeropage	        LAX oper	    A7	2	    3
 * zeropage,Y	    LAX oper,Y	    B7	2	    4
 * absolute	        LAX oper	    AF	3	    4
 * absolut,Y	    LAX oper,Y	    BF	3	    4*
 * (indirect,X)	    LAX (oper,X)	A3	2	    6
 * (indirect),Y	    LAX (oper),Y	B3	2	    5*
*/
#[named]
fn lax<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    c.regs.a = b;
    c.regs.x = b;
    set_zn_flags(c, b);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LDA - Load Accumulator
 *
 * A,Z,N = M
 *
 * Loads a byte of memory into the accumulator setting the zero and negative flags as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of A is set
 *
 * addressing	assembler	    opc	bytes	cycles
 * immediate	LDA #oper	    A9	2	    2  
 * zeropage	    LDA oper	    A5	2	    3  
 * zeropage,X	LDA oper,X	    B5	2	    4  
 * absolute	    LDA oper	    AD	3	    4  
 * absolute,X	LDA oper,X	    BD	3	    4*
 * absolute,Y	LDA oper,Y	    B9	3	    4*
 * (indirect,X)	LDA (oper,X)	A1	2	    6  
 * (indirect),Y	LDA (oper),Y	B1	2	    5*
 */
#[named]
fn lda<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;
    c.regs.a = b;

    set_zn_flags(c, c.regs.a);

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LDX - Load X Register
 *
 * X,Z,N = M
 *
 * Loads a byte of memory into the X register setting the zero and negative flags as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if X = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of X is set
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	LDX #oper	A2	2	    2  
 * zeropage	    LDX oper	A6	2	    3  
 * zeropage,Y	LDX oper,Y	B6	2	    4  
 * absolute	    LDX oper	AE	3	    4  
 * absolute,Y	LDX oper,Y	BE	3	    4*
 */
#[named]
fn ldx<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;
    c.regs.x = b;

    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LDY - Load Y Register
 *
 * Y,Z,N = M
 *
 * Loads a byte of memory into the Y register setting the zero and negative flags as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if Y = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of Y is set
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	LDY #oper	A0	2	2  
 * zeropage	LDY oper	A4	2	3  
 * zeropage,X	LDY oper,X	B4	2	4  
 * absolute	LDY oper	AC	3	4  
 * absolute,X	LDY oper,X	BC	3	4*
 */
#[named]
fn ldy<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;
    c.regs.y = b;

    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LSR - Logical Shift Right
 *
 * A,C,Z,N = A/2 or M,C,Z,N = M/2
 *
 * Each of the bits in A or M is shift one place to the right. The bit that was in bit 0 is shifted into the carry flag. Bit 7 is set to zero.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set to contents of old bit 0
 * Z	Zero Flag	Set if result = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * accumulator	LSR A	    4A	1	    2  
 * zeropage	    LSR oper	46	2	    5  
 * zeropage,X	LSR oper,X	56	2	    6  
 * absolute	    LSR oper	4E	3	    6  
 * absolute,X	LSR oper,X	5E	3	    7  
 */
#[named]
fn lsr<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let mut b = A::load(c, d, tgt, rw_bp_triggered)?;

    // save bit 0 in the carry
    c.set_carry_flag(b & 1 != 0);

    // lsr
    b >>= 1;

    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, rw_bp_triggered, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * LXA (LAX immediate)
 *
 * Store * AND oper in A and X
 *
 * Highly unstable, involves a 'magic' constant, see ANE
 *
 * (A OR CONST) AND oper -> A -> X
 *
 * N	Z	C	I	D	V
 * +	+	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	LXA #oper	AB	2	2  	††
*/
#[named]
fn lxa<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    let k = 0xff;
    let res: u8 = (c.regs.a | k) & b;
    c.regs.x = res;
    c.regs.a = res;

    set_zn_flags(c, res);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * NOP - No Operation
 *
 * The NOP instruction causes no changes to the processor other than the normal incrementing of the program counter to the next instruction.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    NOP	        EA	1	    2     
*/

#[named]
fn nop<A: AddressingMode>(
    c: &mut Cpu,
    _: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    _: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // noop, do nothing ...
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * ORA - Logical Inclusive OR
 *
 * A,Z,N = A|M
 * An inclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 set
 *
 * addressing	assembler	    opc	bytes	cycles
 * immediate	ORA #oper	    09	2	    2  
 * zeropage	    ORA oper	    05	2	    3  
 * zeropage,X	ORA oper,X	    15	2	    4  
 * absolute	    ORA oper	    0D	3	    4  
 * absolute,X	ORA oper,X	    1D	3	    4*
 * absolute,Y	ORA oper,Y	    19	3	    4*
 * (indirect,X)	ORA (oper,X)	01	2	    6  
 * (indirect),Y	ORA (oper),Y	11	2	    5*
 */
#[named]
fn ora<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;
    c.regs.a |= b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * PHA - Push Accumulator
 *
 * Pushes a copy of the accumulator on to the stack.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    PHA	        48	1	    3  
 */
#[named]
fn pha<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    push_byte(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn php<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

/**
 * SBC - Subtract with Carry
 *
 * A,Z,C,N = A-M-(1-C)
 *
 * This instruction subtracts the contents of a memory location to the accumulator together with the not of the carry bit.
 * If overflow occurs the carry bit is clear, this enables multiple byte subtraction to be performed.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Clear if overflow in bit 7
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Set if sign bit is incorrect
 * N	Negative Flag	Set if bit 7 set
 *
 * SBC implementation converted from c code, taken from https://github.com/DavidBuchanan314/6502-emu/blob/master/6502.c
 */
#[named]
fn sbc<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

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
            .wrapping_add(c.is_carry_set() as u8);
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

/**
* SBX (undoc)
* CMP and DEX at once, sets flags like CMP
*
* (A AND X) - oper -> X
*
* N	Z	C	I	D	V
* +	+	+	-	-	-
*/
#[named]
fn sbx<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // read operand
    let b = A::load(c, d, tgt, rw_bp_triggered)?;

    // and
    c.regs.a = c.regs.a & c.regs.x;

    // cmp
    c.regs.a = c.regs.a.wrapping_sub(b);
    c.set_carry_flag(c.regs.a >= b);
    c.set_zero_flag(c.regs.a == b);

    c.set_negative_flag(utils::is_signed(c.regs.x));
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn sec<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    // store A in memory
    A::store(c, d, tgt, rw_bp_triggered, c.regs.a)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}

#[named]
fn stx<A: AddressingMode>(
    c: &mut Cpu,
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (_, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
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
    d: &Debugger,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    rw_bp_triggered: bool,
    quiet: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    if !quiet {
        debug_out_opcode::<A>(c, function_name!())?;
    }
    if decode_only {
        // perform decode only, no execution
        return Ok((A::len(), 0));
    }

    panic!("*** NOT IMPLEMENTED ***");
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }))
}
