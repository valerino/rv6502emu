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
 * Permission is hereby grante free of charge, to any person obtaining a copy of
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
use crate::cpu::cpu_error::{CpuError, CpuErrorType};
use crate::cpu::CpuFlags;
use crate::cpu::{Cpu, CpuOperation, CpuType, Vectors};
use crate::utils;
use ::function_name::named;

use lazy_static::*;

lazy_static! {
/**
 * the 6502 256 opcodes table (includes undocumented)
 *
 * for clarity, each Vec element is a tuple defined as this (each element is named including return values):
 *
 * (< fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool) -> Result<(instr_size:i8, effective_cycles:usize), CpuError>, in_cycles: usize, add_extra_cycle:bool, opcode_name: &'static str, addressing_mode: AddressingModeId) >)
 *
 * all the opcodes info are taken from, in no particular order :
 *
 * - https://www.masswerk.at/6502/6502_instruction_set.html
 * - https://problemkaputt.de/2k6specs.htm#cpu65xxmicroprocessor
 * - http://www.oxyron.de/html/opcodes02.html
 * - http://6502.org/tutorials/65c02opcodes.html
 * - http://www.obelisk.me.uk/6502/reference.html (WARNING: ASL, LSR, ROL, ROR info is wrong! flag Z is set when RESULT=0, not when A=0. i fixed this in functions comments.)
 * - [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
 */

pub(crate) static ref OPCODE_MATRIX: Vec<( fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool) -> Result<(i8, usize), CpuError>, usize, bool, &'static str, AddressingModeId)> =
    vec![
        // 0x0 - 0xf
        (brk::<ImpliedAddressing>, 7, false, "brk",Imp),
        (ora::<XIndirectAddressing>, 6, false, "ora",Xin),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (slo::<XIndirectAddressing>, 8, false, "slo",Xin),
        (nop::<ZeroPageAddressing>, 3, false, "nop",Zpg),
        (ora::<ZeroPageAddressing>, 3, false, "ora",Zpg),
        (asl::<ZeroPageAddressing>, 5, false, "asl",Zpg),
        (slo::<ZeroPageAddressing>, 5, false, "slo",Zpg),
        (php::<ImpliedAddressing>, 3, false, "php",Imp),
        (ora::<ImmediateAddressing>, 2, false, "ora",Imm),
        (asl::<AccumulatorAddressing>, 2, false, "asl",Acc),
        (anc::<ImmediateAddressing>, 2, false, "anc",Imm),
        (nop::<AbsoluteAddressing>, 4, false, "nop",Abs),
        (ora::<AbsoluteAddressing>, 4, false, "ora",Abs),
        (asl::<AbsoluteAddressing>, 6, false, "asl",Abs),
        (slo::<AbsoluteAddressing>, 6, false, "slo",Abs),

        // 0x10 - 0x1f
        (bpl::<RelativeAddressing>, 2, true, "bpl",Rel),
        (ora::<IndirectYAddressing>, 5, true, "ora",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (slo::<IndirectYAddressing>, 8, false, "slo",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (ora::<ZeroPageXAddressing>, 4, false, "ora",Zpx),
        (asl::<ZeroPageXAddressing>, 6, false, "asl",Zpx),
        (slo::<ZeroPageXAddressing>, 6, false, "slo",Zpx),
        (clc::<ImpliedAddressing>, 2, false, "clc",Imp),
        (ora::<AbsoluteYAddressing>, 4, true, "ora",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (slo::<AbsoluteYAddressing>, 7, false, "slo",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (ora::<AbsoluteXAddressing>, 4, true, "ora",Abx),
        (asl::<AbsoluteXAddressing>, 7, false, "asl",Abx),
        (slo::<AbsoluteXAddressing>, 7, false, "slo",Abx),

        // 0x20 - 0x2f
        (jsr::<AbsoluteAddressing>, 6, false, "jsr",Abs),
        (and::<XIndirectAddressing>, 6, false, "and",Xin),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (rla::<XIndirectAddressing>, 8, false, "rla",Xin),
        (bit::<ZeroPageAddressing>, 3, false, "bit",Zpg),
        (and::<ZeroPageAddressing>, 3, false, "and",Zpg),
        (rol::<ZeroPageAddressing>, 5, false, "rol",Zpg),
        (rla::<ZeroPageAddressing>, 5, false, "rla",Zpg),
        (plp::<ImpliedAddressing>, 4, false, "plp",Imp),
        (and::<ImmediateAddressing>, 2, false, "and",Imm),
        (rol::<AccumulatorAddressing>, 2, false, "rol",Acc),
        (anc::<ImmediateAddressing>, 2, false, "anc",Imm),
        (bit::<AbsoluteAddressing>, 4, false, "bit",Abs),
        (and::<AbsoluteAddressing>, 4, false, "and",Abs),
        (rol::<AbsoluteAddressing>, 6, false, "rol",Abs),
        (rla::<AbsoluteAddressing>, 6, false, "rla",Abs),

        // 0x30 - 0x3f
        (bmi::<RelativeAddressing>, 2, true, "bmi",Rel),
        (and::<IndirectYAddressing>, 5, true, "and",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (rla::<IndirectYAddressing>, 8, false, "rla",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (and::<ZeroPageXAddressing>, 4, false, "and",Zpx),
        (rol::<ZeroPageXAddressing>, 6, false, "rol",Zpx),
        (rla::<ZeroPageXAddressing>, 6, false, "rla",Zpx),
        (sec::<ImpliedAddressing>, 2, false, "sec",Imp),
        (and::<AbsoluteYAddressing>, 4, true, "and",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (rla::<AbsoluteYAddressing>, 7, false, "rla",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (and::<AbsoluteXAddressing>, 4, true, "and",Abx),
        (rol::<AbsoluteXAddressing>, 7, false, "rol",Abx),
        (rla::<AbsoluteXAddressing>, 7, false, "rla",Abx),

        // 0x40 - 0x4f
        (rti::<ImpliedAddressing>, 6, false, "rti",Imp),
        (eor::<XIndirectAddressing>, 6, false, "eor",Xin),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (sre::<XIndirectAddressing>, 8, false, "sre",Xin),
        (nop::<ZeroPageAddressing>, 3, false, "nop",Zpg),
        (eor::<ZeroPageAddressing>, 3, false, "eor",Zpg),
        (lsr::<ZeroPageAddressing>, 5, false, "lsr",Zpg),
        (sre::<ZeroPageAddressing>, 5, false, "sre",Zpg),
        (pha::<ImpliedAddressing>, 3, false, "pha",Imp),
        (eor::<ImmediateAddressing>, 2, false, "eor",Imm),
        (lsr::<AccumulatorAddressing>, 2, false, "lsr",Acc),
        (alr::<ImmediateAddressing>, 2, false, "alr",Imm),
        (jmp::<AbsoluteAddressing>, 3, false, "jmp",Abs),
        (eor::<AbsoluteAddressing>, 4, false, "eor",Abs),
        (lsr::<AbsoluteAddressing>, 6, false, "lsr",Abs),
        (sre::<AbsoluteAddressing>, 6, false, "sre",Abs),

        // 0x50 - 0x5f
        (bvc::<RelativeAddressing>, 2, true, "bvc",Rel),
        (eor::<IndirectYAddressing>, 5, true, "eor",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (sre::<IndirectYAddressing>, 8, false, "sre",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (eor::<ZeroPageXAddressing>, 4, false, "eor",Zpx),
        (lsr::<ZeroPageXAddressing>, 6, false, "lsr",Zpx),
        (sre::<ZeroPageXAddressing>, 6, false, "sre",Zpx),
        (cli::<ImpliedAddressing>, 2, false, "cli",Imp),
        (eor::<AbsoluteYAddressing>, 4, true, "eor",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (sre::<AbsoluteYAddressing>, 7, false, "sre",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (eor::<AbsoluteXAddressing>, 4, true, "eor",Abx),
        (lsr::<AbsoluteXAddressing>, 7, false, "lsr",Abx),
        (sre::<AbsoluteXAddressing>, 7, false, "sre",Abx),

        // 0x60 - 0x6f
        (rts::<ImpliedAddressing>, 6, false, "rts",Imp),
        (adc::<XIndirectAddressing>, 6, false, "adc",Xin),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (rra::<XIndirectAddressing>, 8, false, "rra",Xin),
        (nop::<ZeroPageAddressing>, 3, false, "nop",Zpg),
        (adc::<ZeroPageAddressing>, 3, false, "adc",Zpg),
        (ror::<ZeroPageAddressing>, 5, false, "ror",Zpg),
        (rra::<ZeroPageAddressing>, 5, false, "rra",Zpg),
        (pla::<ImpliedAddressing>, 4, false, "pla",Imp),
        (adc::<ImmediateAddressing>, 2, true, "adc",Imm),
        (ror::<AccumulatorAddressing>, 2, false, "ror",Acc),
        (arr::<ImmediateAddressing>, 2, false, "arr",Imm),
        (jmp::<IndirectAddressing>, 5, false, "jmp",Ind),
        (adc::<AbsoluteAddressing>, 4, false, "adc",Abs),
        (ror::<AbsoluteAddressing>, 6, false, "ror",Abs),
        (rra::<AbsoluteAddressing>, 6, false, "rra",Abs),

        // 0x70 - 0x7f
        (bvs::<RelativeAddressing>, 2, true, "bvs",Rel),
        (adc::<IndirectYAddressing>, 5, true, "adc",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (rra::<IndirectYAddressing>, 8, false, "rra",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (adc::<ZeroPageXAddressing>, 4, false, "adc",Zpx),
        (ror::<ZeroPageXAddressing>, 6, false, "ror",Zpx),
        (rra::<ZeroPageXAddressing>, 6, false, "rra",Zpx),
        (sei::<ImpliedAddressing>, 2, false, "sei",Imp),
        (adc::<AbsoluteYAddressing>, 4, true, "adc",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (rra::<AbsoluteYAddressing>, 7, false, "rra",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (adc::<AbsoluteXAddressing>, 4, true, "adc",Abx),
        (ror::<AbsoluteXAddressing>, 7, false, "ror",Abx),
        (rra::<AbsoluteXAddressing>, 7, false, "rra",Abx),

        // 0x80 - 0x8f
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (sta::<XIndirectAddressing>, 6, false, "sta",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (sax::<XIndirectAddressing>, 6, false, "sax",Xin),
        (sty::<ZeroPageAddressing>, 3, false, "sty",Zpg),
        (sta::<ZeroPageAddressing>, 3, false, "sta",Zpg),
        (stx::<ZeroPageAddressing>, 3, false, "stx",Zpg),
        (sax::<ZeroPageAddressing>, 3, false, "sax",Zpg),
        (dey::<ImpliedAddressing>, 2, false, "dey",Imp),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (txa::<ImpliedAddressing>, 2, false, "txa",Imp),
        (xaa::<ImmediateAddressing>, 2, false, "xaa",Imm),
        (sty::<AbsoluteAddressing>, 4, false, "sty",Abs),
        (sta::<AbsoluteAddressing>, 4, false, "sta",Abs),
        (stx::<AbsoluteAddressing>, 4, false, "stx",Abs),
        (sax::<AbsoluteAddressing>, 4, false, "sax",Abs),

        // 0x90 - 0x9f
        (bcc::<RelativeAddressing>, 2, true, "bcc",Rel),
        (sta::<IndirectYAddressing>, 6, false, "sta",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (ahx::<IndirectYAddressing>, 6, false, "ahx",Iny),
        (sty::<ZeroPageXAddressing>, 4, false, "sty",Zpx),
        (sta::<ZeroPageXAddressing>, 4, false, "sta",Zpx),
        (stx::<ZeroPageYAddressing>, 4, false, "stx",Zpy),
        (sax::<ZeroPageYAddressing>, 4, false, "sax",Zpy),
        (tya::<ImpliedAddressing>, 2, false, "tya",Imp),
        (sta::<AbsoluteYAddressing>, 5, false, "sta",Aby),
        (txs::<ImpliedAddressing>, 2, false, "txs",Imp),
        (tas::<AbsoluteYAddressing>, 5, false, "tas",Aby),
        (shy::<AbsoluteXAddressing>, 5, false, "shy",Abx),
        (sta::<AbsoluteXAddressing>, 5, false, "sta",Abx),
        (shx::<AbsoluteYAddressing>, 5, false, "shx",Aby),
        (ahx::<AbsoluteYAddressing>, 5, false, "ahx",Aby),

        // 0xa0 - 0xaf
        (ldy::<ImmediateAddressing>, 2, false, "ldy",Imm),
        (lda::<XIndirectAddressing>, 6, false, "lda",Xin),
        (ldx::<ImmediateAddressing>, 2, false, "ldx",Imm),
        (lax::<XIndirectAddressing>, 6, false, "lax",Xin),
        (ldy::<ZeroPageAddressing>, 3, false, "ldy",Zpg),
        (lda::<ZeroPageAddressing>, 3, false, "lda",Zpg),
        (ldx::<ZeroPageAddressing>, 3, false, "ldx",Zpg),
        (lax::<ZeroPageAddressing>, 3, false, "lax",Zpg),
        (tay::<ImpliedAddressing>, 2, false, "tay",Imp),
        (lda::<ImmediateAddressing>, 2, false, "lda",Imm),
        (tax::<ImpliedAddressing>, 2, false, "tax",Imp),
        (lax::<ImmediateAddressing>, 2, false, "lxa",Imm),
        (ldy::<AbsoluteAddressing>, 4, false, "ldy",Abs),
        (lda::<AbsoluteAddressing>, 4, false, "lda",Abs),
        (ldx::<AbsoluteAddressing>, 4, false, "ldx",Abs),
        (lax::<AbsoluteAddressing>, 4, false, "lax",Abs),

        // 0xb0 - 0xbf
        (bcs::<RelativeAddressing>, 2, true, "bcs",Rel),
        (lda::<IndirectYAddressing>, 5, true, "lda",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (lax::<IndirectYAddressing>, 5, true, "lax",Iny),
        (ldy::<ZeroPageXAddressing>, 4, false, "ldy",Zpx),
        (lda::<ZeroPageXAddressing>, 4, false, "lda",Zpx),
        (ldx::<ZeroPageYAddressing>, 4, false, "ldx",Zpy),
        (lax::<ZeroPageYAddressing>, 4, false, "lax",Zpy),
        (clv::<ImpliedAddressing>, 2, false, "clv",Imp),
        (lda::<AbsoluteYAddressing>, 4, true, "lda",Aby),
        (tsx::<ImpliedAddressing>, 2, false, "tsx",Imp),
        (las::<AbsoluteYAddressing>, 4, true, "las",Aby),
        (ldy::<AbsoluteXAddressing>, 4, true, "ldy",Abx),
        (lda::<AbsoluteXAddressing>, 4, true, "lda",Abx),
        (ldx::<AbsoluteYAddressing>, 4, true, "ldx",Aby),
        (lax::<AbsoluteYAddressing>, 4, true, "lax",Aby),

        // 0xc0 - 0xcf
        (cpy::<ImmediateAddressing>, 2, false, "cpy",Imm),
        (cmp::<XIndirectAddressing>, 6, false, "cmp",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (dcp::<XIndirectAddressing>, 8, false, "dcp",Xin),
        (cpy::<ZeroPageAddressing>, 3, false, "cpy",Zpg),
        (cmp::<ZeroPageAddressing>, 3, false, "cmp",Zpg),
        (dec::<ZeroPageAddressing>, 5, false, "dec",Zpg),
        (dcp::<ZeroPageAddressing>, 5, false, "dcp",Zpg),
        (iny::<ImpliedAddressing>, 2, false, "iny",Imp),
        (cmp::<ImmediateAddressing>, 2, false, "cmp",Imm),
        (dex::<ImpliedAddressing>, 2, false, "dex",Imp),
        (sbx::<ImmediateAddressing>, 2, false, "sbx",Imm),
        (cpy::<AbsoluteAddressing>, 4, false, "cpy",Abs),
        (cmp::<AbsoluteAddressing>, 4, false, "cmp",Abs),
        (dec::<AbsoluteAddressing>, 6, false, "dec",Abs),
        (dcp::<AbsoluteAddressing>, 6, false, "dcp",Abs),

        // 0xd0 - 0xdf
        (bne::<RelativeAddressing>, 2, true, "bne",Rel),
        (cmp::<IndirectYAddressing>, 5, true, "cmp",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (dcp::<IndirectYAddressing>, 8, false, "dcp",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (cmp::<ZeroPageXAddressing>, 4, false, "cmp",Zpx),
        (dec::<ZeroPageXAddressing>, 6, false, "dec",Zpx),
        (dcp::<ZeroPageXAddressing>, 6, false, "dcp",Zpx),
        (cld::<ImpliedAddressing>, 2, false, "cld",Imp),
        (cmp::<AbsoluteYAddressing>, 4, true, "cmp",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (dcp::<AbsoluteYAddressing>, 7, false, "dcp",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (cmp::<AbsoluteXAddressing>, 4, true, "cmp",Abx),
        (dec::<AbsoluteXAddressing>, 7, false, "dec",Abx),
        (dcp::<AbsoluteXAddressing>, 7, false, "dcp",Abx),

        // 0xe0 - 0xef
        (cpx::<ImmediateAddressing>, 2, false, "cpx",Imm),
        (sbc::<XIndirectAddressing>, 6, false, "sbc",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (isc::<XIndirectAddressing>, 8, false, "isc",Xin),
        (cpx::<ZeroPageAddressing>, 3, false, "cpx",Zpg),
        (sbc::<ZeroPageAddressing>, 3, false, "sbc",Zpg),
        (inc::<ZeroPageAddressing>, 5, false, "inc",Zpg),
        (isc::<ZeroPageAddressing>, 5, false, "isc",Zpg),
        (inx::<ImpliedAddressing>, 2, false, "inx",Imp),
        (sbc::<ImmediateAddressing>, 2, false, "sbc",Imm),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (sbc::<ImmediateAddressing>, 2, false, "sbc",Imm),
        (cpx::<AbsoluteAddressing>, 4, false, "cpx",Abs),
        (sbc::<AbsoluteAddressing>, 4, false, "sbc",Abs),
        (inc::<AbsoluteAddressing>, 6, false, "inc",Abs),
        (isc::<AbsoluteAddressing>, 6, false, "isc",Abs),

        // 0xf0 - 0xff
        (beq::<RelativeAddressing>, 2, true, "beq",Rel),
        (sbc::<IndirectYAddressing>, 5, true, "sbc",Iny),
        (kil::<ImpliedAddressing>, 0, false, "kil",Imp),
        (isc::<IndirectYAddressing>, 8, false, "isc",Iny),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (sbc::<ZeroPageXAddressing>, 4, false, "sbc",Zpx),
        (inc::<ZeroPageXAddressing>, 6, false, "inc",Zpx),
        (isc::<ZeroPageXAddressing>, 6, false, "isc",Zpx),
        (sed::<ImpliedAddressing>, 2, false, "sed",Imp),
        (sbc::<AbsoluteYAddressing>, 4, true, "sbc",Aby),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (isc::<AbsoluteYAddressing>, 7, false, "isc",Aby),
        (nop::<AbsoluteXAddressing>, 4, true, "nop",Abx),
        (sbc::<AbsoluteXAddressing>, 4, true, "sbc",Abx),
        (inc::<AbsoluteXAddressing>, 7, false, "inc",Abx),
        (isc::<AbsoluteXAddressing>, 7, false, "isc",Abx),
    ];

/// 65C02 opcode table, same as above with the 65C02 differences.
pub(crate) static ref OPCODE_MATRIX_65C02: Vec<( fn(c: &mut Cpu, in_cycles: usize, extra_cycle_on_page_crossing: bool) -> Result<(i8, usize), CpuError>, usize, bool, &'static str, AddressingModeId)> =
    vec![
        // 0x0 - 0xf
        (brk::<ImpliedAddressing>, 7, false, "brk",Imp),
        (ora::<XIndirectAddressing>, 6, false, "ora",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (tsb::<ZeroPageAddressing>, 5, false, "tsb",Zpg),
        (ora::<ZeroPageAddressing>, 3, false, "ora",Zpg),
        (asl::<ZeroPageAddressing>, 5, false, "asl",Zpg),
        (rmb0::<ZeroPageAddressing>, 5, false, "rmb0",Zpg),
        (php::<ImpliedAddressing>, 3, false, "php",Imp),
        (ora::<ImmediateAddressing>, 2, false, "ora",Imm),
        (asl::<AccumulatorAddressing>, 2, false, "asl",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (tsb::<AbsoluteAddressing>, 6, false, "tsb",Abs),
        (ora::<AbsoluteAddressing>, 4, false, "ora",Abs),
        (asl::<AbsoluteAddressing>, 6, false, "asl",Abs),
        (bbr0::<ZeroPageRelativeAddressing>, 5, false, "bbr0",Zpr),

        // 0x10 - 0x1f
        (bpl::<RelativeAddressing>, 2, true, "bpl",Rel),
        (ora::<IndirectYAddressing>, 5, true, "ora",Iny),
        (ora::<IndirectZeroPageAddressing>, 5, false, "ora",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (trb::<ZeroPageAddressing>, 5, false, "trb",Zpg),
        (ora::<ZeroPageXAddressing>, 4, false, "ora",Zpx),
        (asl::<ZeroPageXAddressing>, 6, false, "asl",Zpx),
        (rmb1::<ZeroPageAddressing>, 5, false, "rmb1",Zpg),
        (clc::<ImpliedAddressing>, 2, false, "clc",Imp),
        (ora::<AbsoluteYAddressing>, 4, true, "ora",Aby),
        (inc::<AccumulatorAddressing>, 2, false, "inc",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (trb::<AbsoluteAddressing>, 6, false, "trb",Abs),
        (ora::<AbsoluteXAddressing>, 4, true, "ora",Abx),
        (asl::<AbsoluteXAddressing>, 6, true, "asl",Abx),
        (bbr1::<ZeroPageRelativeAddressing>, 5, false, "bbr1",Zpr),

        // 0x20 - 0x2f
        (jsr::<AbsoluteAddressing>, 6, false, "jsr",Abs),
        (and::<XIndirectAddressing>, 6, false, "and",Abx),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (bit::<ZeroPageAddressing>, 3, false, "bit",Zpg),
        (and::<ZeroPageAddressing>, 3, false, "and",Zpg),
        (rol::<ZeroPageAddressing>, 5, false, "rol",Zpg),
        (rmb2::<ZeroPageAddressing>, 5, false, "rmb2",Zpg),
        (plp::<ImpliedAddressing>, 4, false, "plp",Imp),
        (and::<ImmediateAddressing>, 2, false, "and",Imm),
        (rol::<AccumulatorAddressing>, 2, false, "rol",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (bit::<AbsoluteAddressing>, 4, false, "bit",Abs),
        (and::<AbsoluteAddressing>, 4, false, "and",Abs),
        (rol::<AbsoluteAddressing>, 6, false, "rol",Abs),
        (bbr2::<ZeroPageRelativeAddressing>, 5, false, "bbr2",Zpr),

        // 0x30 - 0x3f
        (bmi::<RelativeAddressing>, 2, true, "bmi",Rel),
        (and::<IndirectYAddressing>, 5, true, "and",Iny),
        (and::<IndirectZeroPageAddressing>, 5, false, "and",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (bit::<ZeroPageXAddressing>, 4, false, "bit",Zpx),
        (and::<ZeroPageXAddressing>, 4, false, "and",Zpx),
        (rol::<ZeroPageXAddressing>, 6, false, "rol",Zpx),
        (rmb3::<ZeroPageAddressing>, 5, false, "rmb3",Zpg),
        (sec::<ImpliedAddressing>, 2, false, "sec",Imp),
        (and::<AbsoluteYAddressing>, 4, true, "and",Aby),
        (dec::<AccumulatorAddressing>, 2, false, "dec",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (bit::<AbsoluteXAddressing>, 4, true, "bit",Abx),
        (and::<AbsoluteXAddressing>, 4, true, "and",Abx),
        (rol::<AbsoluteXAddressing>, 6, true, "rol",Abx),
        (bbr3::<ZeroPageRelativeAddressing>, 5, false, "bbr3",Zpr),

        // 0x40 - 0x4f
        (rti::<ImpliedAddressing>, 6, false, "rti",Imp),
        (eor::<XIndirectAddressing>, 6, false, "eor",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<ZeroPageAddressing>, 3, false, "nop",Zpg),
        (eor::<ZeroPageAddressing>, 3, false, "eor",Zpg),
        (lsr::<ZeroPageAddressing>, 5, false, "lsr",Zpg),
        (rmb4::<ZeroPageAddressing>, 5, false, "rmb4",Zpg),
        (pha::<ImpliedAddressing>, 3, false, "pha",Imp),
        (eor::<ImmediateAddressing>, 2, false, "eor",Imm),
        (lsr::<AccumulatorAddressing>, 2, false, "lsr",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (jmp::<AbsoluteAddressing>, 3, false, "jmp",Abs),
        (eor::<AbsoluteAddressing>, 4, false, "eor",Abs),
        (lsr::<AbsoluteAddressing>, 6, false, "lsr",Abs),
        (bbr4::<ZeroPageRelativeAddressing>, 5, false, "bbr4",Zpr),

        // 0x50 - 0x5f
        (bvc::<RelativeAddressing>, 2, true, "bvc",Rel),
        (eor::<IndirectYAddressing>, 5, true, "eor",Iny),
        (eor::<IndirectZeroPageAddressing>, 5, false, "eor",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (eor::<ZeroPageXAddressing>, 4, false, "eor",Zpx),
        (lsr::<ZeroPageXAddressing>, 6, false, "lsr",Zpx),
        (rmb5::<ZeroPageAddressing>, 5, false, "rmb5",Zpg),
        (cli::<ImpliedAddressing>, 2, false, "cli",Imp),
        (eor::<AbsoluteYAddressing>, 4, true, "eor",Aby),
        (phy::<ImpliedAddressing>, 3, false, "phy",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<AbsoluteAddressing>, 8, false, "nop",Abs),
        (eor::<AbsoluteXAddressing>, 4, true, "eor",Abx),
        (lsr::<AbsoluteXAddressing>, 6, true, "lsr",Abx),
        (bbr5::<ZeroPageRelativeAddressing>, 5, false, "bbr5",Zpr),

        // 0x60 - 0x6f
        (rts::<ImpliedAddressing>, 6, false, "rts",Imp),
        (adc::<XIndirectAddressing>, 6, false, "adc",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (stz::<ZeroPageAddressing>, 3, false, "stz",Zpg),
        (adc::<ZeroPageAddressing>, 3, false, "adc",Zpg),
        (ror::<ZeroPageAddressing>, 5, false, "ror",Zpg),
        (rmb6::<ZeroPageAddressing>, 5, false, "rmb6",Zpg),
        (pla::<ImpliedAddressing>, 4, false, "pla",Imp),
        (adc::<ImmediateAddressing>, 2, true, "adc",Imm),
        (ror::<AccumulatorAddressing>, 2, false, "ror",Acc),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (jmp::<IndirectAddressing>, 6, false, "jmp",Ind),
        (adc::<AbsoluteAddressing>, 4, false, "adc",Abs),
        (ror::<AbsoluteAddressing>, 6, false, "ror",Abs),
        (bbr6::<ZeroPageRelativeAddressing>, 5, false, "bbr6",Zpr),

        // 0x70 - 0x7f
        (bvs::<RelativeAddressing>, 2, true, "bvs",Rel),
        (adc::<IndirectYAddressing>, 5, true, "adc",Iny),
        (adc::<IndirectZeroPageAddressing>, 5, false, "adc",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (stz::<ZeroPageXAddressing>, 4, false, "stz",Zpx),
        (adc::<ZeroPageXAddressing>, 4, false, "adc",Zpx),
        (ror::<ZeroPageXAddressing>, 6, false, "ror",Zpx),
        (rmb7::<ZeroPageAddressing>, 5, false, "rmb7",Zpg),
        (sei::<ImpliedAddressing>, 2, false, "sei",Imp),
        (adc::<AbsoluteYAddressing>, 4, true, "adc",Aby),
        (ply::<ImpliedAddressing>, 4, false, "ply",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (jmp::<AbsoluteIndirectXAddressing>, 6, false, "jmp",Aix),
        (adc::<AbsoluteXAddressing>, 4, true, "adc",Abx),
        (ror::<AbsoluteXAddressing>, 7, true, "ror",Abx),
        (bbr7::<ZeroPageRelativeAddressing>, 5, false, "bbr7",Zpr),

        // 0x80 - 0x8f
        (bra::<RelativeAddressing>, 3, true, "bra",Rel),
        (sta::<XIndirectAddressing>, 6, false, "sta",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (sty::<ZeroPageAddressing>, 3, false, "sty",Zpg),
        (sta::<ZeroPageAddressing>, 3, false, "sta",Zpg),
        (stx::<ZeroPageAddressing>, 3, false, "stx",Zpg),
        (smb0::<ZeroPageAddressing>, 5, false, "smb0",Zpg),
        (dey::<ImpliedAddressing>, 2, false, "dey",Imp),
        (bit::<ImmediateAddressing>, 2, false, "bit",Imm),
        (txa::<ImpliedAddressing>, 2, false, "txa",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (sty::<AbsoluteAddressing>, 4, false, "sty",Abs),
        (sta::<AbsoluteAddressing>, 4, false, "sta",Abs),
        (stx::<AbsoluteAddressing>, 4, false, "stx",Abs),
        (bbs0::<ZeroPageRelativeAddressing>, 5, false, "bbs0",Zpr),

        // 0x90 - 0x9f
        (bcc::<RelativeAddressing>, 2, true, "bcc",Rel),
        (sta::<IndirectYAddressing>, 6, false, "sta",Iny),
        (sta::<IndirectZeroPageAddressing>, 5, false, "kil",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (sty::<ZeroPageXAddressing>, 4, false, "sty",Zpx),
        (sta::<ZeroPageXAddressing>, 4, false, "sta",Zpx),
        (stx::<ZeroPageYAddressing>, 4, false, "stx",Zpy),
        (smb1::<ZeroPageAddressing>, 5, false, "smb1",Zpg),
        (tya::<ImpliedAddressing>, 2, false, "tya",Imp),
        (sta::<AbsoluteYAddressing>, 5, false, "sta",Aby),
        (txs::<ImpliedAddressing>, 2, false, "txs",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (stz::<AbsoluteAddressing>, 4, false, "stz",Abs),
        (sta::<AbsoluteXAddressing>, 5, false, "sta",Abx),
        (stz::<AbsoluteXAddressing>, 5, false, "stz",Abx),
        (bbs1::<ZeroPageRelativeAddressing>, 5, false, "bbs1",Zpr),

        // 0xa0 - 0xaf
        (ldy::<ImmediateAddressing>, 2, false, "ldy",Imm),
        (lda::<XIndirectAddressing>, 6, false, "lda",Xin),
        (ldx::<ImmediateAddressing>, 2, false, "ldx",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (ldy::<ZeroPageAddressing>, 3, false, "ldy",Zpg),
        (lda::<ZeroPageAddressing>, 3, false, "lda",Zpg),
        (ldx::<ZeroPageAddressing>, 3, false, "ldx",Zpg),
        (smb2::<ZeroPageAddressing>, 5, false, "smb2",Zpg),
        (tay::<ImpliedAddressing>, 2, false, "tay",Imp),
        (lda::<ImmediateAddressing>, 2, false, "lda",Imm),
        (tax::<ImpliedAddressing>, 2, false, "tax",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (ldy::<AbsoluteAddressing>, 4, false, "ldy",Abs),
        (lda::<AbsoluteAddressing>, 4, false, "lda",Abs),
        (ldx::<AbsoluteAddressing>, 4, false, "ldx",Abs),
        (bbs2::<ZeroPageRelativeAddressing>, 5, false, "bbs2",Zpr),

        // 0xb0 - 0xbf
        (bcs::<RelativeAddressing>, 2, true, "bcs",Rel),
        (lda::<IndirectYAddressing>, 5, true, "lda",Iny),
        (lda::<IndirectZeroPageAddressing>, 5, false, "lda",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (ldy::<ZeroPageXAddressing>, 4, false, "ldy",Zpx),
        (lda::<ZeroPageXAddressing>, 4, false, "lda",Zpx),
        (ldx::<ZeroPageYAddressing>, 4, false, "ldx",Zpy),
        (smb3::<ZeroPageAddressing>, 5, false, "smb3",Zpg),
        (clv::<ImpliedAddressing>, 2, false, "clv",Imp),
        (lda::<AbsoluteYAddressing>, 4, true, "lda",Aby),
        (tsx::<ImpliedAddressing>, 2, false, "tsx",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (ldy::<AbsoluteXAddressing>, 4, true, "ldy",Abx),
        (lda::<AbsoluteXAddressing>, 4, true, "lda",Abx),
        (ldx::<AbsoluteYAddressing>, 4, true, "ldx",Aby),
        (bbs3::<ZeroPageRelativeAddressing>, 5, false, "bbs3",Zpr),

        // 0xc0 - 0xcf
        (cpy::<ImmediateAddressing>, 2, false, "cpy",Imm),
        (cmp::<XIndirectAddressing>, 6, false, "cmp",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (cpy::<ZeroPageAddressing>, 3, false, "cpy",Zpg),
        (cmp::<ZeroPageAddressing>, 3, false, "cmp",Zpg),
        (dec::<ZeroPageAddressing>, 5, false, "dec",Zpg),
        (smb4::<ZeroPageAddressing>, 5, false, "smb4",Zpg),
        (iny::<ImpliedAddressing>, 2, false, "iny",Imp),
        (cmp::<ImmediateAddressing>, 2, false, "cmp",Imm),
        (dex::<ImpliedAddressing>, 2, false, "dex",Imp),
        (wai::<ImpliedAddressing>, 3, false, "wai",Imp),
        (cpy::<AbsoluteAddressing>, 4, false, "cpy",Abs),
        (cmp::<AbsoluteAddressing>, 4, false, "cmp",Abs),
        (dec::<AbsoluteAddressing>, 6, false, "dec",Abs),
        (bbs4::<ZeroPageRelativeAddressing>, 5, false, "bbs4",Zpr),

        // 0xd0 - 0xdf
        (bne::<RelativeAddressing>, 2, true, "bne",Rel),
        (cmp::<IndirectYAddressing>, 5, true, "cmp",Iny),
        (cmp::<IndirectZeroPageAddressing>, 5, false, "cmp",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (cmp::<ZeroPageXAddressing>, 4, false, "cmp",Zpx),
        (dec::<ZeroPageXAddressing>, 6, false, "dec",Zpx),
        (smb5::<ZeroPageAddressing>, 5, false, "smb5",Zpg),
        (cld::<ImpliedAddressing>, 2, false, "cld",Imp),
        (cmp::<AbsoluteYAddressing>, 4, true, "cmp",Aby),
        (phx::<ImpliedAddressing>, 3, false, "phx",Imp),
        (stp::<ImpliedAddressing>, 3, false, "stp",Imp),
        (nop::<AbsoluteAddressing>, 4, true, "nop",Abs),
        (cmp::<AbsoluteXAddressing>, 4, true, "cmp",Abx),
        (dec::<AbsoluteXAddressing>, 7, false, "dec",Abx),
        (bbs5::<ZeroPageRelativeAddressing>, 5, false, "bbs5",Zpr),

        // 0xe0 - 0xef
        (cpx::<ImmediateAddressing>, 2, false, "cpx",Imm),
        (sbc::<XIndirectAddressing>, 6, false, "sbc",Xin),
        (nop::<ImmediateAddressing>, 2, false, "nop",Imm),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (cpx::<ZeroPageAddressing>, 3, false, "cpx",Zpg),
        (sbc::<ZeroPageAddressing>, 3, false, "sbc",Zpg),
        (inc::<ZeroPageAddressing>, 5, false, "inc",Zpg),
        (smb6::<ZeroPageAddressing>, 5, false, "smb6",Zpg),
        (inx::<ImpliedAddressing>, 2, false, "inx",Imp),
        (sbc::<ImmediateAddressing>, 2, false, "sbc",Imm),
        (nop::<ImpliedAddressing>, 2, false, "nop",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (cpx::<AbsoluteAddressing>, 4, false, "cpx",Abs),
        (sbc::<AbsoluteAddressing>, 4, false, "sbc",Abs),
        (inc::<AbsoluteAddressing>, 6, false, "inc",Abs),
        (bbs6::<ZeroPageRelativeAddressing>, 5, false, "bbs6",Zpr),

        // 0xf0 - 0xff
        (beq::<RelativeAddressing>, 2, true, "beq",Rel),
        (sbc::<IndirectYAddressing>, 5, true, "sbc",Iny),
        (sbc::<IndirectZeroPageAddressing>, 5, false, "sbc",Izp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<ZeroPageXAddressing>, 4, false, "nop",Zpx),
        (sbc::<ZeroPageXAddressing>, 4, false, "sbc",Zpx),
        (inc::<ZeroPageXAddressing>, 6, false, "inc",Zpx),
        (smb7::<ZeroPageAddressing>, 5, false, "smb7",Zpg),
        (sed::<ImpliedAddressing>, 2, false, "sed",Imp),
        (sbc::<AbsoluteYAddressing>, 4, true, "sbc",Aby),
        (plx::<ImpliedAddressing>, 4, false, "plx",Imp),
        (nop::<ImpliedAddressing>, 1, false, "nop",Imp),
        (nop::<AbsoluteAddressing>, 4, true, "nop",Abs),
        (sbc::<AbsoluteXAddressing>, 4, true, "sbc",Abx),
        (inc::<AbsoluteXAddressing>, 7, false, "inc",Abx),
        (bbs7::<ZeroPageRelativeAddressing>, 5, false, "bbs7",Zpr),
    ];
 }

/**
 * prints the 6502 opcode table, for debugging ....
 */
#[allow(dead_code)]
pub fn print_opcode_table() {
    let mut c = 0;
    for (i, (_, _, _, name, id)) in OPCODE_MATRIX.iter().enumerate() {
        print!(
            "{}0x{:02x}={}({})",
            if i & 0xf == 2
                || i & 0xf == 3
                || i & 0xf == 4
                || i & 0xf == 7
                || i & 0xf == 0xb
                || i & 0xf == 0xc
                || i & 0xf == 0xf
            {
                // indicates specific 65c02 opcode, or change to standard 6502 opcode in 65c02
                // http://6502.org/tutorials/65c02opcodes.html
                "**"
            } else {
                ""
            },
            i,
            name,
            id
        );
        if c == 15 {
            print!("\n");
            c = 0;
        } else {
            print!(",");
            c += 1;
        }
    }
    print!("\n");
}

/**
 * helper to set Z and N flags in one shot, depending on val
 */
fn set_zn_flags(c: &mut Cpu, val: u8) {
    c.set_cpu_flags(CpuFlags::Z, val == 0);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(val));
}

/**
 * push byte on the stack
 */
pub(super) fn push_byte(c: &mut Cpu, b: u8) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    let addr = 0x100 + c.regs.s as usize;
    mem.write_byte(addr, b)?;
    c.regs.s = c.regs.s.wrapping_sub(1);
    /* // handle breakpoint
    if d.is_some() {
        d.unwrap().handle_rw_breakpoint(c, addr as u16, BreakpointType::WRITE)?
    }
    */
    // call callback if any
    c.call_callback(addr as u16, b, 1, CpuOperation::Write);
    Ok(())
}

/**
 * pop byte off the stack
 */
fn pop_byte(c: &mut Cpu) -> Result<u8, CpuError> {
    let mem = c.bus.get_memory();
    c.regs.s = c.regs.s.wrapping_add(1);
    let addr = 0x100 + c.regs.s as usize;
    let b = mem.read_byte(addr)?;

    /*
    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::READ)?
    }
    */

    // call callback if any
    c.call_callback(addr as u16, b, 1, CpuOperation::Read);
    Ok(b)
}

/**
 * pop word off the stack
 */
fn pop_word_le(c: &mut Cpu) -> Result<u16, CpuError> {
    let mem = c.bus.get_memory();
    c.regs.s = c.regs.s.wrapping_add(2);
    let addr = 0x100 + (c.regs.s - 1) as usize;

    let w = mem.read_word_le(addr)?;

    /*
    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::READ)?
    }
    */
    // call callback if any
    c.call_callback(addr as u16, (w & 0xff) as u8, 2, CpuOperation::Read);

    Ok(w)
}

/**
 * push word on the stack
 */
pub(super) fn push_word_le(c: &mut Cpu, w: u16) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    let addr = 0x100 + (c.regs.s - 1) as usize;
    mem.write_word_le(addr, w)?;
    c.regs.s = c.regs.s.wrapping_sub(2);

    /*
    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::WRITE)?
    }
    */
    // call callback if any
    c.call_callback(addr as u16, (w & 0xff) as u8, 2, CpuOperation::Write);
    Ok(())
}

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
#[named]
fn adc<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, mut cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;

    let b = A::load(c, tgt)?;

    // perform the addition (regs.a+b+C)
    let mut sum: u16;
    if c.is_cpu_flag_set(CpuFlags::D) {
        if c.cpu_type == CpuType::WDC65C02 {
            // one extra cycle in decimal mode
            cycles += 1;
        }

        // bcd
        sum = ((c.regs.a as u16) & 0x0f)
            .wrapping_add((b as u16) & 0x0f)
            .wrapping_add(c.is_cpu_flag_set(CpuFlags::C) as u16);
        if sum >= 10 {
            sum = (sum.wrapping_sub(10)) | 0x10;
        }
        sum = sum
            .wrapping_add((c.regs.a as u16) & 0xf0)
            .wrapping_add((b as u16) & 0xf0);
        if sum > 0x9f {
            sum = sum.wrapping_add(0x60);
        }
    } else {
        // normal
        sum = (c.regs.a as u16)
            .wrapping_add(b as u16)
            .wrapping_add(c.is_cpu_flag_set(CpuFlags::C) as u16);
    }
    // set flags
    c.set_cpu_flags(CpuFlags::C, sum > 0xff);
    let o = ((c.regs.a as u16) ^ sum) & ((b as u16) ^ sum) & 0x80;
    c.set_cpu_flags(CpuFlags::V, o != 0);
    c.regs.a = (sum & 0xff) as u8;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
}

/**
 * AHX (undoc) (aka SHA, aka AXA)
 *
 * Stores A AND X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is droppe page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;

    // get msb from target address
    let mut h = (tgt >> 8) as u8;

    // add 1 on msb when page crossing
    // [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    if extra_cycle_on_page_crossing {
        h = h.wrapping_add(1);
    }

    // A & X & (H + 1)
    let res = c.regs.a & c.regs.x & h.wrapping_add(1);

    // store
    A::store(c, tgt, res)?;

    Ok((A::len(), cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // and (preserve flags, n and z are set in lfr)
    let prev_p = c.regs.p;
    and::<A>(c, 0, _extra_cycle_on_page_crossing)?;
    c.regs.p = prev_p;

    // lsr A
    lsr::<AccumulatorAddressing>(c, 0, false)?;

    Ok((A::len(), in_cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // and
    and::<A>(c, in_cycles, _extra_cycle_on_page_crossing)?;
    c.set_cpu_flags(CpuFlags::C, utils::is_signed(c.regs.a));
    Ok((A::len(), in_cycles))
}

/**
* AND - Logical AND
*
* A,Z,N = A&M
*
* A logical AND is performe bit by bit, on the accumulator contents using the contents of a byte of memory.
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // A AND M -> A
    c.regs.a = c.regs.a & b;

    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
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
 *
 * implemented according to [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
 */
#[named]
fn arr<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    if !c.is_cpu_flag_set(CpuFlags::D) {
        // and
        and::<A>(c, 0, false)?;

        // ror A
        let prev_a = c.regs.a;
        ror::<AccumulatorAddressing>(c, 0, false)?;

        // set carry and overflow
        c.set_cpu_flags(CpuFlags::C, utils::is_signed(prev_a));
        let is_bit_6_set = prev_a & 0b01000000;
        c.set_cpu_flags(
            CpuFlags::V,
            (is_bit_6_set as i8 ^ utils::is_signed(prev_a) as i8) != 0,
        );
        set_zn_flags(c, c.regs.a);
    } else {
        // decimal
        // and
        and::<A>(c, 0, false)?;
        let and_res = c.regs.a;
        ror::<AccumulatorAddressing>(c, 0, false)?;

        // fix for decimal

        // original C is preserved in N
        c.set_cpu_flags(CpuFlags::N, c.is_cpu_flag_set(CpuFlags::C));

        // Z is set when the ROR produced a zero result
        c.set_cpu_flags(CpuFlags::Z, c.regs.a == 0);

        // V is set when bit 6 of the result was changed by the ROR
        let v = ((c.regs.a ^ and_res) & 0x40) >> 6;
        c.set_cpu_flags(CpuFlags::V, v != 0);

        // fixup for low nibble
        if (and_res & 0xf) + (and_res & 0x1) > 0x5 {
            c.regs.a = (c.regs.a & 0xf0) | ((c.regs.a + 0x6) & 0xf);
        }
        // fixup for high nibble, set carry
        if (and_res & 0xf0) + (and_res & 0x10) > 0x50 {
            c.regs.a = (c.regs.a & 0x0f) | ((c.regs.a + 0x60) & 0xf0);
            c.set_cpu_flags(CpuFlags::C, true);
        } else {
            c.set_cpu_flags(CpuFlags::C, false);
        }
    }
    Ok((A::len(), in_cycles))
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
 * Z	Zero Flag	Set if result = 0
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;
    c.set_cpu_flags(CpuFlags::C, utils::is_signed(b));

    // shl
    b <<= 1;
    set_zn_flags(c, b);

    // store back
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if !c.is_cpu_flag_set(CpuFlags::C) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    if c.is_cpu_flag_set(CpuFlags::C) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;

    if c.is_cpu_flag_set(CpuFlags::Z) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    let and_res = c.regs.a & b;

    c.set_cpu_flags(CpuFlags::Z, and_res == 0);

    // on 65c02 and immediate mode, N and V are not affected
    if c.cpu_type == CpuType::MOS6502
        || (c.cpu_type == CpuType::WDC65C02 && A::id() != AddressingModeId::Imm)
    {
        c.set_cpu_flags(CpuFlags::N, utils::is_signed(b));
        c.set_cpu_flags(CpuFlags::V, b & 0b01000000 != 0);
    }
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    let b = A::load(c, tgt)?;

    // branch
    if c.is_cpu_flag_set(CpuFlags::N) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);

        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }

        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut cycles = in_cycles;
    let mut taken: bool = false;
    let b = A::load(c, tgt)?;

    // branch
    if !c.is_cpu_flag_set(CpuFlags::Z) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);

        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    let mut cycles = in_cycles;
    let mut taken: bool = false;
    // branch
    if !c.is_cpu_flag_set(CpuFlags::N) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // push pc and p on stack
    push_word_le(c, c.regs.pc + 2)?;

    // push P with U and B set
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    let mut flags = c.regs.p.clone();
    flags.set(CpuFlags::B, true);
    flags.set(CpuFlags::U, true);
    push_byte(c, flags.bits())?;

    if c.cpu_type == CpuType::WDC65C02 {
        // clear the D flag
        // http://6502.org/tutorials/65c02opcodes.html
        c.regs.p.set(CpuFlags::D, false);
    }

    // set I
    c.set_cpu_flags(CpuFlags::I, true);

    // set pc to address contained at irq vector
    let addr = c.bus.get_memory().read_word_le(Vectors::IRQ as usize)?;

    // check for deadlock
    if addr == c.regs.pc {
        return Err(CpuError::new_default(
            CpuErrorType::Deadlock,
            c.regs.pc,
            None,
        ));
    }
    c.processing_ints = true;
    c.regs.pc = addr;
    Ok((0, in_cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;

    if !c.is_cpu_flag_set(CpuFlags::V) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch
    let mut cycles = in_cycles;
    let mut taken: bool = false;

    if c.is_cpu_flag_set(CpuFlags::V) {
        // branch is taken, add another cycle
        cycles += 1;
        taken = true;
        let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // clear carry
    c.set_cpu_flags(CpuFlags::C, false);
    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // clear decimal flag
    c.set_cpu_flags(CpuFlags::D, false);
    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // enable interrupts, clear the flag
    c.set_cpu_flags(CpuFlags::I, false);

    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // clear the overflow flag
    c.set_cpu_flags(CpuFlags::V, false);
    Ok((A::len(), in_cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    let res = c.regs.a.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.a >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.a == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    let res = c.regs.x.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.x >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.x == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    let res = c.regs.y.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.y >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.y == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));

    Ok((A::len(), cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform dec + cmp internally (flags are set according to cmp, so save before)
    let prev_p = c.regs.p;
    dec::<A>(c, 0, false)?;
    c.regs.p = prev_p;
    cmp::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;
    b = b.wrapping_sub(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.x = c.regs.x.wrapping_sub(1);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.y = c.regs.y.wrapping_sub(1);
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles))
}

/**
 * EOR - Exclusive OR
 *
 * A,Z,N = A^M
 *
 * An exclusive OR is performe bit by bit, on the accumulator contents using the contents of a byte of memory.
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    c.regs.a = c.regs.a ^ b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    b = b.wrapping_add(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.x = c.regs.x.wrapping_add(1);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.y = c.regs.y.wrapping_add(1);
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform inc + sbc internally (sbc sets p, preserve carry flag after inc)
    let prev_p = c.regs.p;
    inc::<A>(c, 0, false)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    sbc::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
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
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // check for deadlock
    if tgt == c.regs.pc {
        return Err(CpuError::new_default(
            CpuErrorType::Deadlock,
            c.regs.pc,
            None,
        ));
    }
    // set pc
    c.regs.pc = tgt;

    Ok((0, in_cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // push return address
    push_word_le(c, c.regs.pc.wrapping_add(A::len() as u16).wrapping_sub(1))?;

    // check for deadlock
    if tgt == c.regs.pc {
        return Err(CpuError::new_default(
            CpuErrorType::Deadlock,
            c.regs.pc,
            None,
        ));
    }
    // set pc
    c.regs.pc = tgt;

    Ok((0, in_cycles))
}

/**
 * CPU JAM!
 */
#[named]
fn kil<A: AddressingMode>(
    c: &mut Cpu,
    _in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // this is an invalid opcode and emulation should be halted!

    // invalid !
    let mut e = CpuError::new_default(CpuErrorType::InvalidOpcode, c.regs.pc, None);
    e.address = c.regs.pc as usize;
    Err(e)
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // get operand
    let b = A::load(c, tgt)?;
    let res = b & c.regs.s;

    c.regs.a = res;
    c.regs.x = res;
    c.regs.s = res;
    set_zn_flags(c, res);
    Ok((A::len(), cycles))
}

/**
 * LAX (undoc)
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    c.regs.a = b;
    c.regs.x = b;
    set_zn_flags(c, b);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;
    c.regs.a = b;

    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;
    c.regs.x = b;

    set_zn_flags(c, c.regs.x);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;
    c.regs.y = b;

    set_zn_flags(c, c.regs.y);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    // save bit 0 in the carry
    c.set_cpu_flags(CpuFlags::C, b & 1 != 0);

    // lsr
    b >>= 1;

    set_zn_flags(c, b);

    // store back
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
}

/**
 * LXA (undoc) (aka LAX immediate)
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // N and Z are set according to the value of the accumulator before the instruction executed
    set_zn_flags(c, c.regs.a);

    // we choose $ee as constant as specified in [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    let k = 0xee;
    let res: u8 = (c.regs.a | k) & b;
    c.regs.x = res;
    c.regs.a = res;
    Ok((A::len(), cycles))
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
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // noop, do nothing ...
    Ok((A::len(), in_cycles))
}

/**
 * ORA - Logical Inclusive OR
 *
 * A,Z,N = A|M
 * An inclusive OR is performe bit by bit, on the accumulator contents using the contents of a byte of memory.
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;
    c.regs.a |= b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    push_byte(c, c.regs.a)?;
    Ok((A::len(), in_cycles))
}

/**
 * PHP - Push Processor Status
 *
 * Pushes a copy of the status flags on to the stack.
 *
 * The status register will be pushed with the break flag and bit 5 set to 1.
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
 * implied	    PHP	        08	1	    3  
 */
#[named]
fn php<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // ensure B and U(ndefined) are set to 1
    let mut flags = c.regs.p.clone();
    flags.set(CpuFlags::U, true);
    flags.set(CpuFlags::B, true);
    push_byte(c, flags.bits())?;
    Ok((A::len(), in_cycles))
}

/**
 * PLA - Pull Accumulator
 *
 * Pulls an 8 bit value from the stack and into the accumulator. The zero and negative flags are set as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of A is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    PLA	        68	1	    4  
 */
#[named]
fn pla<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.a = pop_byte(c)?;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles))
}

/**
 * PLP - Pull Processor Status
 *
 * Pulls an 8 bit value from the stack and into the processor flags. The flags will take on new states as determined by the value pulled.
 *
 * The status register will be pulled with the break flag and bit 5 ignored.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Set from stack
 * Z	Zero Flag	Set from stack
 * I	Interrupt Disable	Set from stack
 * D	Decimal Mode Flag	Set from stack
 * B	Break Command	Set from stack
 * V	Overflow Flag	Set from stack
 * N	Negative Flag	Set from stack
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    PLP	        28	1	    4  
 */
#[named]
fn plp<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let popped_flags = pop_byte(c)?;
    c.regs.p = CpuFlags::from_bits(popped_flags).unwrap();

    // ensure flag Unused is set and B is unset
    c.set_cpu_flags(CpuFlags::B, false);
    c.set_cpu_flags(CpuFlags::U, true);
    /*
    if c.irq_pending {
        if !c.is_cpu_flag_set(CpuFlags::I) {
            // we'll trigger an irq right after
            c.must_trigger_irq = true;
        }
    }
    */
    Ok((A::len(), in_cycles))
}

/**
 * RLA (undoc) (aka RLN)
 *
 * ROL oper + AND oper
 *
 * M = C <- [76543210] <- C, A AND M -> A
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * zeropage	RLA oper	27	2	5
 * zeropage,X	RLA oper,X	37	2	6
 * absolute	RLA oper	2F	3	6
 * absolut,X	RLA oper,X	3F	3	7
 * absolut,Y	RLA oper,Y	3B	3	7
 * (indirect,X)	RLA (oper,X)	23	2	8
 * (indirect),Y	RLA (oper),Y	33	2	8
 */

#[named]
fn rla<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform rol + and internally
    let prev_p = c.regs.p;
    rol::<A>(c, 0, false)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);
    // n and z are set according to AND
    and::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
}

/**
 * ROL - Rotate Left
 *
 * Move each of the bits in either A or M one place to the left.
 *
 * Bit 0 is filled with the current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
 *
 * Processor Status after use:
 *
 *  C	Carry Flag	Set to contents of old bit 7
 * Z	Zero Flag	Set if result = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of the result is set
 *
 * addressing	assembler	opc	bytes	cycles
 * accumulator	ROL A	    2A	1	    2  
 * zeropage	    ROL oper	26	2	    5  
 * zeropage,X	ROL oper,X	36	2	    6  
 * absolute	    ROL oper	2E	3	    6  
 * absolute,X	ROL oper,X	3E	3	    7
 */
#[named]
fn rol<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    // save current carry
    let carry = c.is_cpu_flag_set(CpuFlags::C);

    // carry = bit 7
    c.set_cpu_flags(CpuFlags::C, utils::is_signed(b));

    b <<= 1;

    // bit 0 = previous C
    if carry {
        b |= 0b00000001
    } else {
        b &= 0b11111110
    }

    // store back
    A::store(c, tgt, b)?;
    set_zn_flags(c, b);
    Ok((A::len(), cycles))
}

/**
 * ROR - Rotate Right
 *
 * Move each of the bits in either A or M one place to the right.
 *
 * Bit 7 is filled with the current value of the carry flag whilst the old bit 0 becomes the new carry flag value.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	        Set to contents of old bit 0
 * Z	Zero Flag	        Set if result = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	    Not affected
 * V	Overflow Flag	    Not affected
 * N	Negative Flag	    Set if bit 7 of the result is set
 */
#[named]
fn ror<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    // save current carry
    let carry = c.is_cpu_flag_set(CpuFlags::C);

    // save current bit 0
    let is_bit_0_set = b & 1;

    // shr
    b >>= 1;

    // set bit 7 and C accordingly
    if carry {
        b |= 0b10000000;
    } else {
        b &= 0b01111111;
    }
    c.set_cpu_flags(CpuFlags::C, is_bit_0_set == 1);

    // store back
    A::store(c, tgt, b)?;
    set_zn_flags(c, b);
    Ok((A::len(), cycles))
}

/**
 * RRA (undoc)
 *
 * ROR oper + ADC oper
 *
 * M = C -> [76543210] -> C, A + M + C -> A, C
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	+
 *
 * addressing	assembler	opc	bytes	cycles
 * zeropage	RRA oper	67	2	5
 * zeropage,X	RRA oper,X	77	2	6
 * absolute	RRA oper	6F	3	6
 * absolut,X	RRA oper,X	7F	3	7
 * absolut,Y	RRA oper,Y	7B	3	7
 * (indirect,X)	RRA (oper,X)	63	2	8
 * (indirect),Y	RRA (oper),Y	73	2	8
 */
#[named]
fn rra<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform ror + adc internally
    let prev_p = c.regs.p;
    ror::<A>(c, 0, false)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // all other flags are set by adc
    adc::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
}

/**
* RTI - Return from Interrupt
*
* The RTI instruction is used at the end of an interrupt processing routine.
*
* It pulls the processor flags from the stack followed by the program counter.
*
* The status register is pulled with the break flag and bit 5 ignored. Then PC is pulled from the stack.
*
* Processor Status after use:
*
* C	Carry Flag	Set from stack
* Z	Zero Flag	Set from stack
* I	Interrupt Disable	Set from stack
* D	Decimal Mode Flag	Set from stack
* B	Break Command	Set from stack
* V	Overflow Flag	Set from stack
* N	Negative Flag	Set from stack
*
* addressing	assembler	opc	bytes	cycles
* implied	    RTI	        40	1	    6
*/
#[named]
fn rti<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let popped_flags = pop_byte(c)?;
    c.regs.p = CpuFlags::from_bits(popped_flags).unwrap();

    // ensure flag Unused is set and B is unset
    c.set_cpu_flags(CpuFlags::B, false);
    c.set_cpu_flags(CpuFlags::U, true);

    // pull pc
    c.regs.pc = pop_word_le(c)?;

    // apply fix if neede and anyway reset the flag.
    //c.regs.pc = c.regs.pc.wrapping_add(c.fix_pc_rti as u16);
    //c.fix_pc_rti = 0;
    c.processing_ints = false;
    println!("returning from RTI at pc=${:04x}", c.regs.pc);
    Ok((0, in_cycles))
}

/**
 * RTS - Return from Subroutine
 *
 * The RTS instruction is used at the end of a subroutine to return to the calling routine.
 *
 * It pulls the program counter (minus one) from the stack.
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
 * implied	    RTS 	    60	1	    6  
 */
#[named]
fn rts<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.pc = pop_word_le(c)?.wrapping_add(1);
    Ok((0, in_cycles))
}

/**
 * SAX (undoc) (aka AXS, aka AAX)
 *
 * A and X are put on the bus at the same time (resulting effectively in an AND operation) and stored in M
 *
 * A AND X -> M
 *
 * N	Z	C	I	D	V
 * -	-	-	-	-	-
 *
 * addressing	assembler	    opc	bytes	cycles
 * zeropage	    SAX oper	    87	2	    3
 * zeropage,Y	SAX oper,Y	    97	2	    4
 * absolute	    SAX oper	    8F	3	    4
 * (indirect,X)	SAX (oper,X)	83	2	    6
 */
#[named]
fn sax<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = c.regs.a & c.regs.x;
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
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

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // get target_address
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut cycles = in_cycles;

    let b = A::load(c, tgt)?;

    // perform non-bcd subtraction (regs.a-b-1+C)
    let sub: u16 = (c.regs.a as u16)
        .wrapping_sub(b as u16)
        .wrapping_sub(1)
        .wrapping_add(c.is_cpu_flag_set(CpuFlags::C) as u16);
    let o = ((c.regs.a as u16) ^ sub) & ((c.regs.a as u16) ^ (b as u16)) & 0x80;
    c.set_cpu_flags(CpuFlags::V, o != 0);

    if c.is_cpu_flag_set(CpuFlags::D) {
        if c.cpu_type == CpuType::WDC65C02 {
            // one extra cycle in decimal mode
            cycles += 1;
        }

        // bcd
        let mut lo: u8 = (c.regs.a & 0x0f)
            .wrapping_sub(b & 0x0f)
            .wrapping_sub(1)
            .wrapping_add(c.is_cpu_flag_set(CpuFlags::C) as u8);
        let mut hi: u8 = (c.regs.a >> 4).wrapping_sub(b >> 4);
        if lo & 0x10 != 0 {
            lo = lo.wrapping_sub(6);
            hi = hi.wrapping_sub(1);
        }
        if hi & 0x10 != 0 {
            hi = hi.wrapping_sub(6);
        }
        c.regs.a = (hi << 4) | (lo & 0xf);
    } else {
        // normal
        c.regs.a = (sub & 0xff) as u8;
    }
    c.set_cpu_flags(CpuFlags::C, sub < 0x100);
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), cycles))
}

/**
* SBX (undoc) (aka AXS, SAX)
*
* CMP and DEX at once, sets flags like CMP
*
* (A AND X) - oper -> X
*
* N	Z	C	I	D	V
* +	+	+	-	-	-
*
* addressing	assembler	opc	bytes	cycles
* immediate	    SBX #oper	CB	2	    2
*/
#[named]
fn sbx<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // and
    let and_res = c.regs.a & c.regs.x;

    // cmp
    c.regs.x = and_res.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.a >= b);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), cycles))
}

/**
 * SEC - Set Carry Flag
 *
 * C = 1
 *
 * Set the carry flag to one.
 *
 * C	Carry Flag	Set to 1
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    SEC	        38	1	    2  
 */
#[named]
fn sec<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // set carry
    c.set_cpu_flags(CpuFlags::C, true);
    Ok((A::len(), in_cycles))
}

/**
 * SED - Set Decimal Flag
 *
 * D = 1
 *
 * Set the carry flag to one.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Set to 1
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    SED	        F8	1	    2  
 */
#[named]
fn sed<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // set decimal flag
    c.set_cpu_flags(CpuFlags::D, true);
    Ok((A::len(), in_cycles))
}

/**
 * SEI - Set Interrupt Disable Flag
 *
 * I = 1
 *
 * Set the carry flag to one.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Set to 1
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    SED	        78	1	    2  
 */
#[named]
fn sei<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // disable interrupts
    c.set_cpu_flags(CpuFlags::I, true);
    Ok((A::len(), in_cycles))
}

/**
 * SHX (undoc) (aka A11, SXA, XAS)
 *
 * Stores X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is droppe page boundary crossings may not work
 * (with the high-byte of the value used as the high-byte of the address)
 *
 * X AND (H+1) -> M
 *
 * N	Z	C	I	D	V
 * -	-	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * absolut,Y	SHX oper,Y	9E	3	5  	†
*/
#[named]
fn shx<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // get msb from target address
    let mut h = (tgt >> 8) as u8;

    // add 1 on msb when page crossing
    // [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    if extra_cycle_on_page_crossing {
        h = h.wrapping_add(1);
    }

    // X & (H + 1)
    let res = c.regs.x & h.wrapping_add(1);

    // store
    A::store(c, tgt, res)?;
    Ok((A::len(), cycles))
}

/**
 * SHY (undoc) (aka A11, SYA, SAY)
 *
 * Stores Y AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is droppe page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
 *
 * Y AND (H+1) -> M
 *
 * N	Z	C	I	D	V
 * -	-	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * absolut,X	SHY oper,X	9C	3	    5  	†
*/
#[named]
fn shy<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // get msb from target address
    let mut h = (tgt >> 8) as u8;

    // add 1 on msb when page crossing
    // [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    if extra_cycle_on_page_crossing {
        h = h.wrapping_add(1);
    }

    // Y & (H + 1)
    let res = c.regs.y & h.wrapping_add(1);

    // store
    A::store(c, tgt, res)?;
    Ok((A::len(), cycles))
}

/**
 * SLO (undoc) (aka ASO)
 *
 * ASL oper + ORA oper
 *
 * M = C <- [76543210] <- 0, A OR M -> A
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	    opc	bytes	cycles
 * zeropage	    SLO oper	    07	2	    5
 * zeropage,X	SLO oper,X	    17	2	    6
 * absolute	    SLO oper	    0F	3	    6
 * absolut,X	SLO oper,X	    1F	3	    7
 * absolut,Y	SLO oper,Y	    1B	3	    7
 * (indirect,X)	SLO (oper,X)	03	2	    8
 * (indirect),Y	SLO (oper),Y	13	2	    8
*/
#[named]
fn slo<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform asl + ora internally
    let prev_p = c.regs.p;
    asl::<A>(c, 0, false)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // other flags are set by ora
    ora::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
}

/**
 * SRE (undoc) (aka LSE)
 *
 * LSR oper + EOR oper
 *
 * M = 0 -> [76543210] -> C, A EOR M -> A
 *
 * N	Z	C	I	D	V
 * +	+	+	-	-	-
 *
 * addressing	assembler	    opc	bytes	cycles
 * zeropage	    SRE oper	    47	2	    5
 * zeropage,X	SRE oper,X	    57	2	    6
 * absolute	    SRE oper	    4F	3	    6
 * absolut,X	SRE oper,X	    5F	3	    7
 * absolut,Y	SRE oper,Y	    5B	3	    7
 * (indirect,X)	SRE (oper,X)	43	2	    8
 * (indirect),Y	SRE (oper),Y	53	2	    8  
 */
#[named]
fn sre<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // perform lsr + eor internally
    let prev_p = c.regs.p;
    lsr::<A>(c, 0, false)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // other flags are set by eor
    eor::<A>(c, 0, false)?;
    Ok((A::len(), in_cycles))
}

/**
 * STA - Store Accumulator
 *
 * M = A
 *
 * Stores the contents of the accumulator into memory.
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
 */
#[named]
fn sta<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?; // store A in memory
    A::store(c, tgt, c.regs.a)?;
    Ok((A::len(), cycles))
}

/**
 * STX - Store X Register
 *
 * M = X
 *
 * Stores the contents of the X register into memory.
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
 */
#[named]
fn stx<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // store X in memory
    A::store(c, tgt, c.regs.x)?;
    Ok((A::len(), cycles))
}

/**
 * STY - Store Y Register
 *
 * M = Y
 *
 * Stores the contents of the Y register into memory.
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
 */
#[named]
fn sty<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // store y in memory
    A::store(c, tgt, c.regs.y)?;
    Ok((A::len(), cycles))
}

/**
 * TAS (undoc) (aka XAS, SHS)
 *
 * Puts A AND X in SP and stores A AND X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is droppe page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
 *
 * A AND X -> SP, A AND X AND (H+1) -> M
 *
 * N	Z	C	I	D	V
 * -	-	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * absolut,Y	TAS oper,Y	9B	3	5  	†
*/
#[named]
fn tas<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // get msb from target address
    let mut h = (tgt >> 8) as u8;

    // add 1 on msb when page crossing
    // [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    if extra_cycle_on_page_crossing {
        h = h.wrapping_add(1);
    }

    // set sp
    c.regs.s = c.regs.a & c.regs.x;
    let res = c.regs.s & h.wrapping_add(1);

    // store
    A::store(c, tgt, res)?;
    Ok((A::len(), cycles))
}

/**
 * TAX - Transfer Accumulator to X
 *
 * X = A
 *
 * Copies the current contents of the accumulator into the X register and sets the zero and negative flags as appropriate.
 *
 * Processor Status after use:
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
 * implied	    TAX	        AA	1	    2  
 */
#[named]
fn tax<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.x = c.regs.a;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles))
}

/**
 * TAY - Transfer Accumulator to Y
 *
 * Y = A
 * Copies the current contents of the accumulator into the Y register and sets the zero and negative flags as appropriate.
 *
 * Processor Status after use:
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
 * implied	    TAY	        A8	1	    2  
*/
#[named]
fn tay<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.y = c.regs.a;
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles))
}

/**
 * TSX - Transfer Stack Pointer to X
 *
 * X = S
 *
 * Copies the current contents of the stack register into the X register and sets the zero and negative flags as appropriate.
 *
 * Processor Status after use:
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
 * implied	    TSX	        BA	1	    2
 */
#[named]
fn tsx<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.x = c.regs.s;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles))
}

/**
 * TXA - Transfer X to Accumulator
 *
 * A = X
 *
 * Copies the current contents of the X register into the accumulator and sets the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of A is set
 *
 * addressing	assembler	opc	bytes	cycles
 * implied	    TXA	        8A	1	    2  
*/
#[named]
fn txa<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.a = c.regs.x;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles))
}

/**
 * TXS - Transfer X to Stack Pointer
 *
 * S = X
 *
 * Copies the current contents of the X register into the stack register.
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
 * implied	    TXS	        9A	1	    2  
 */
#[named]
fn txs<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.s = c.regs.x;
    Ok((A::len(), in_cycles))
}

/**
 * TYA - Transfer Y to Accumulator
 *
 * A = Y
 *
 * Copies the current contents of the Y register into the accumulator and sets the zero and negative flags as appropriate.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of A is set
 *
 * addressing	assembler	opc	    bytes	cycles
 * implied	    TYA	        98	    1	    2  
 */
#[named]
fn tya<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.a = c.regs.y;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles))
}

/**
 * XAA (undoc) (aka ANE)
 *
 * AND X + AND oper
 *
 * Highly unstable, do not use.
 *
 * A base value in A is determined based on the contets of A and a constant, which may be typically $00, $ff, $ee, etc.
 * The value of this constant depends on temerature, the chip series, and maybe other factors, as well.
 *
 * In order to eliminate these uncertaincies from the equation, use either 0 as the operand or a value of $FF in the accumulator.
 *
 * (A OR CONST) AND X AND oper -> A
 *
 * N	Z	C	I	D	V
 * +	+	-	-	-	-
 *
 * addressing	assembler	opc	bytes	cycles
 * immediate	ANE #oper	8B	2	    2  	††
 */

#[named]
fn xaa<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // N and Z are set according to the value of the accumulator before the instruction executed
    set_zn_flags(c, c.regs.a);

    // we choose $ef as constant as specified in [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    let k = 0xef;
    let res: u8 = (c.regs.a | k) & c.regs.x & b;
    c.regs.a = res;
    Ok((A::len(), cycles))
}

/**
 * 65c02 only opcodes following
 */

fn bbr_bbs_internal<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    bit: i8,
    name: &str,
    is_bbr: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // get byte to test
    let to_test_addr = A::load(c, c.regs.pc.wrapping_add(1))?;
    let to_test = A::load(c, to_test_addr as u16)?;

    let taken: bool;
    if is_bbr {
        taken = (to_test & (1 << bit)) == 0;
    } else {
        taken = (to_test & (1 << bit)) != 0;
    }
    if taken {
        // branch is taken
        let (mut new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
        new_pc = new_pc.wrapping_add(1);
        // check for deadlock
        if new_pc == c.regs.pc {
            return Err(CpuError::new_default(
                CpuErrorType::Deadlock,
                c.regs.pc,
                None,
            ));
        }
        c.regs.pc = new_pc;
    }
    Ok((if taken { 0 } else { A::len() }, in_cycles))
}

/**
 * BBR - Branch on Bit Reset
 *
 * This a actually a set of 8 instructions. Each tests a specific bit of a byte held on zero page and causes a branch if the bit is reset (0). For example:
 *
 * BBR7 VALUE, ISPOSITIVE  
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
 */
#[named]
fn bbr0<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        0,
        function_name!(),
        true,
    )
}

#[named]
fn bbr1<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        1,
        function_name!(),
        true,
    )
}

#[named]
fn bbr2<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        2,
        function_name!(),
        true,
    )
}

#[named]
fn bbr3<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        3,
        function_name!(),
        true,
    )
}

#[named]
fn bbr4<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        4,
        function_name!(),
        true,
    )
}

#[named]
fn bbr5<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(c, in_cycles, extra_cycle_on_page_crossing, 5, "bbr5", true)
}

#[named]
fn bbr6<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        6,
        function_name!(),
        true,
    )
}

#[named]
fn bbr7<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        7,
        function_name!(),
        true,
    )
}

#[named]
fn bbs0<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        0,
        function_name!(),
        false,
    )
}

#[named]
fn bbs1<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        1,
        function_name!(),
        false,
    )
}

#[named]
fn bbs2<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        2,
        function_name!(),
        false,
    )
}

#[named]
fn bbs3<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        3,
        function_name!(),
        false,
    )
}

#[named]
fn bbs4<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        4,
        function_name!(),
        false,
    )
}

#[named]
fn bbs5<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        5,
        function_name!(),
        false,
    )
}

#[named]
fn bbs6<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        6,
        function_name!(),
        false,
    )
}

#[named]
fn bbs7<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        7,
        function_name!(),
        false,
    )
}

fn rmb_smb_internal<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    bit: i8,
    name: &str,
    is_rmb: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    if is_rmb {
        // reset bit
        b &= !(1 << bit);
    } else {
        // set bit
        b |= 1 << bit;
    }

    // write
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
}

/**
 * RMB - Reset Memory Bit
 *
 * This a actually a set of 8 instructions. Each resets a specific bit of a byte held on zero page. For example:
 *
 * RMB5 VALUE
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
 */
#[named]
fn rmb0<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        0,
        function_name!(),
        true,
    )
}

#[named]
fn rmb1<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        1,
        function_name!(),
        true,
    )
}

#[named]
fn rmb2<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        2,
        function_name!(),
        true,
    )
}

#[named]
fn rmb3<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        3,
        function_name!(),
        true,
    )
}

#[named]
fn rmb4<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        4,
        function_name!(),
        true,
    )
}

#[named]
fn rmb5<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        5,
        function_name!(),
        true,
    )
}

#[named]
fn rmb6<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        6,
        function_name!(),
        true,
    )
}

#[named]
fn rmb7<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        7,
        function_name!(),
        true,
    )
}

#[named]
fn smb0<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        0,
        function_name!(),
        false,
    )
}

#[named]
fn smb1<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        1,
        function_name!(),
        false,
    )
}

#[named]
fn smb2<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        2,
        function_name!(),
        false,
    )
}

#[named]
fn smb3<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        3,
        function_name!(),
        false,
    )
}

#[named]
fn smb4<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        4,
        function_name!(),
        false,
    )
}

#[named]
fn smb5<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        5,
        function_name!(),
        false,
    )
}

#[named]
fn smb6<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        6,
        function_name!(),
        false,
    )
}

#[named]
fn smb7<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    rmb_smb_internal::<A>(
        c,
        in_cycles,
        extra_cycle_on_page_crossing,
        7,
        function_name!(),
        false,
    )
}

/**
 * BRA - Branch Always
 *
 * Adds the relative displacement to the program counter to cause a branch to a new location.
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
 */
#[named]
fn bra<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let b = A::load(c, tgt)?;

    // branch is always taken
    let (new_pc, _) = addressing_modes::get_relative_branch_target(c.regs.pc, b);
    // check for deadlock
    if new_pc == c.regs.pc {
        return Err(CpuError::new_default(
            CpuErrorType::Deadlock,
            c.regs.pc,
            None,
        ));
    }
    c.regs.pc = new_pc;
    Ok((0, in_cycles))
}

/**
 * PHX - Push X Register
 *
 * Pushes a copy of the X register  on to the stack.
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
 */
#[named]
fn phx<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    push_byte(c, c.regs.x)?;
    Ok((A::len(), in_cycles))
}

/**
 * PHY - Push Y Register
 *
 * Pushes a copy of the Y register  on to the stack.
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
 */
#[named]
fn phy<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    push_byte(c, c.regs.y)?;
    Ok((A::len(), in_cycles))
}

/**
 * PLX - Pull X Register
 *
 * Pulls an 8 bit value from the stack and into the X register. The zero and negative flags are set as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if X = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of X is set
 */
#[named]
fn plx<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.x = pop_byte(c)?;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles))
}

/**
 * PLY - Pull Y Register
 *
 * Pulls an 8 bit value from the stack and into the Y register. The zero and negative flags are set as appropriate.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Set if A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Set if bit 7 of A is set
 */
#[named]
fn ply<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    c.regs.y = pop_byte(c)?;
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles))
}

/**
 * STP - Stop
 *
 * The processor halts until a hardware reset is applied.
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	Not affected
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 */
#[named]
fn stp<A: AddressingMode>(
    _c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    // deadlock
    Ok((0, in_cycles))
}

/**
 * STZ - Store Zero
 *
 * M = 0
 *
 * Stores a zero byte value into memory.
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
 */
#[named]
fn stz<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    // store
    A::store(c, tgt, 0)?;
    Ok((A::len(), cycles))
}

/**
 * TRB - Test and Reset Bits
 *
 * Z = M & A
 * M = M & ~A
 *
 * The memory byte is tested to see if it contains any of the bits indicated by the value in the accumulator then the bits are reset in the memory byte.
 *
 * TRB has the same effect on the Z flag that a BIT instruction does.
 * Specifically, it is based on whether the result of a bitwise AND of the accumulator with the contents of the memory location specified in the operand is zero.
 * Also, like BIT, the accumulator is not affected.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	set if M & A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 */
#[named]
fn trb<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;

    let res = (b & c.regs.a) == 0;
    c.set_cpu_flags(CpuFlags::Z, res);
    b &= !(c.regs.a);
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
}

/**
 * TSB - Test and Set Bits
 *
 * Z = M & A
 * M = M | A
 *
 * The memory byte is tested to see if it contains any of the bits indicated by the value in the accumulator then the bits are set in the memory byte.
 *
 * TSB, like TRB, has the same effect on the Z flag that a BIT instruction does.
 * Specifically, it is based on whether the result of a bitwise AND of the accumulator with the contents of the memory location specified in the operand is zero.
 * Also, like BIT (and TRB), the accumulator is not affected.
 *
 * Processor Status after use:
 *
 * C	Carry Flag	Not affected
 * Z	Zero Flag	set if M & A = 0
 * I	Interrupt Disable	Not affected
 * D	Decimal Mode Flag	Not affected
 * B	Break Command	Not affected
 * V	Overflow Flag	Not affected
 * N	Negative Flag	Not affected
 */
#[named]
fn tsb<A: AddressingMode>(
    c: &mut Cpu,

    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let (tgt, cycles) = A::target(c, in_cycles, extra_cycle_on_page_crossing)?;
    let mut b = A::load(c, tgt)?;
    let res = (b & c.regs.a) == 0;
    c.set_cpu_flags(CpuFlags::Z, res);
    b |= c.regs.a;
    A::store(c, tgt, b)?;
    Ok((A::len(), cycles))
}

#[named]
fn wai<A: AddressingMode>(
    c: &mut Cpu,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
) -> Result<(i8, usize), CpuError> {
    let mut len = A::len();
    if !c.must_trigger_irq && !c.must_trigger_nmi {
        // will wait for interrupt
        len = 0;
    }
    Ok((len, in_cycles))
}
