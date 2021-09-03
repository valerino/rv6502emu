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
use crate::cpu::cpu_error::{CpuError, CpuErrorType};
use crate::cpu::debugger::breakpoints::BreakpointType;
use crate::cpu::debugger::Debugger;
use crate::cpu::CpuFlags;
use crate::cpu::{Cpu, CpuOperation, CpuType, Vectors};
use crate::utils;
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
 * the 6502 256 opcodes table (includes undocumented)
 *
 * each opcode gets in input a reference to the Cpu, a reference to the Debugger, the cycles needed to execute the opcode, a boolean to indicate if, on crossing page boundaries, an extra cycles must be added,
 * a boolean to indicate decoding only (no execution, for the disassembler).
 * returns a tuple with the instruction size and the effective elapsed cycles (may include the aferomentioned additional cycle).
 *
 * for clarity, each Vec element is a tuple defined as this (each element is named including return values):
 *
 * (< fn(c: &mut Cpu, d: Option<&Debugger>, opcode_byte: u8, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only: bool) -> Result<(instr_size:i8, out_cycles:usize, repr:Option<String>), CpuError>, d: Option<& Debugger>, in_cycles: usize, add_extra_cycle:bool, mrk: OpcodeMarker) >)
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
pub(crate) static ref OPCODE_MATRIX: Vec<( fn(c: &mut Cpu, d: Option<&Debugger>, opcode_byte: u8, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only: bool) -> Result<(i8, usize, Option<String>), CpuError>, usize, bool, OpcodeMarker)> =
    vec![
        // 0x0 - 0xf
        (brk::<ImpliedAddressing>, 7, false, OpcodeMarker{ name: "brk", id: Imp}),
        (ora::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "ora", id: Xin}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (slo::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "slo", id: Xin}),
        (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}),
        (ora::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ora", id: Zpg}),
        (asl::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "asl", id: Zpg}),
        (slo::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "slo", id: Zpg}),
        (php::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "php", id: Imp}),
        (ora::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ora", id: Imm}),
        (asl::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "asl", id: Acc}),
        (anc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "anc", id: Imm}),
        (nop::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Abs}),
        (ora::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Abs}),
        (asl::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Abs}),
        (slo::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "slo", id: Abs}),

        // 0x10 - 0x1f
        (bpl::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bpl", id: Rel}),
        (ora::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "ora", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (slo::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "slo", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (ora::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Zpx}),
        (asl::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Zpx}),
        (slo::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "slo", id: Zpx}),
        (clc::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clc", id: Imp}),
        (ora::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (slo::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "slo", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (ora::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Abx}),
        (asl::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "asl", id: Abx}),
        (slo::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "slo", id: Abx}),

        // 0x20 - 0x2f
        (jsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "jsr", id: Abs}),
        (and::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "and", id: Xin}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (rla::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "rla", id: Xin}),
        (bit::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "bit", id: Zpg}),
        (and::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "and", id: Zpg}),
        (rol::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rol", id: Zpg}),
        (rla::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rla", id: Zpg}),
        (plp::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "plp", id: Imp}),
        (and::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "and", id: Imm}),
        (rol::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "rol", id: Acc}),
        (anc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "anc", id: Imm}),
        (bit::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "bit", id: Abs}),
        (and::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "and", id: Abs}),
        (rol::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Abs}),
        (rla::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rla", id: Abs}),

        // 0x30 - 0x3f
        (bmi::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bmi", id: Rel}),
        (and::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "and", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (rla::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "rla", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (and::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "and", id: Zpx}),
        (rol::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Zpx}),
        (rla::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rla", id: Zpx}),
        (sec::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sec", id: Imp}),
        (and::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "and", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (rla::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "rla", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (and::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "and", id: Abx}),
        (rol::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rol", id: Abx}),
        (rla::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rla", id: Abx}),

        // 0x40 - 0x4f
        (rti::<ImpliedAddressing>, 6, false, OpcodeMarker{ name: "rti", id: Imp}),
        (eor::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "eor", id: Xin}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (sre::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "sre", id: Xin}),
        (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}),
        (eor::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "eor", id: Zpg}),
        (lsr::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "lsr", id: Zpg}),
        (sre::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "sre", id: Zpg}),
        (pha::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "pha", id: Imp}),
        (eor::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "eor", id: Imm}),
        (lsr::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "lsr", id: Acc}),
        (alr::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "alr", id: Imm}),
        (jmp::<AbsoluteAddressing>, 3, false, OpcodeMarker{ name: "jmp", id: Abs}),
        (eor::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Abs}),
        (lsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Abs}),
        (sre::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "sre", id: Abs}),

        // 0x50 - 0x5f
        (bvc::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bvc", id: Rel}),
        (eor::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "eor", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (sre::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "sre", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (eor::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Zpx}),
        (lsr::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Zpx}),
        (sre::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "sre", id: Zpx}),
        (cli::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cli", id: Imp}),
        (eor::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (sre::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "sre", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (eor::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Abx}),
        (lsr::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "lsr", id: Abx}),
        (sre::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "sre", id: Abx}),

        // 0x60 - 0x6f
        (rts::<ImpliedAddressing>, 6, false, OpcodeMarker{ name: "rts", id: Imp}),
        (adc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "adc", id: Xin}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (rra::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "rra", id: Xin}),
        (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}),
        (adc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "adc", id: Zpg}),
        (ror::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "ror", id: Zpg}),
        (rra::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rra", id: Zpg}),
        (pla::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "pla", id: Imp}),
        (adc::<ImmediateAddressing>, 2, true, OpcodeMarker{ name: "adc", id: Imm}),
        (ror::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "ror", id: Acc}),
        (arr::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "arr", id: Imm}),
        (jmp::<IndirectAddressing>, 5, false, OpcodeMarker{ name: "jmp", id: Ind}),
        (adc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Abs}),
        (ror::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Abs}),
        (rra::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rra", id: Abs}),

        // 0x70 - 0x7f
        (bvs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bvs", id: Rel}),
        (adc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "adc", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (rra::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "rra", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (adc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Zpx}),
        (ror::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Zpx}),
        (rra::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rra", id: Zpx}),
        (sei::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sei", id: Imp}),
        (adc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (rra::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "rra", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (adc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Abx}),
        (ror::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "ror", id: Abx}),
        (rra::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "rra", id: Abx}),

        // 0x80 - 0x8f
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (sta::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (sax::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sax", id: Xin}),
        (sty::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sty", id: Zpg}),
        (sta::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sta", id: Zpg}),
        (stx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "stx", id: Zpg}),
        (sax::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sax", id: Zpg}),
        (dey::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dey", id: Imp}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (txa::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txa", id: Imp}),
        (xaa::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "xaa", id: Imm}),
        (sty::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Abs}),
        (sta::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Abs}),
        (stx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Abs}),
        (sax::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sax", id: Abs}),

        // 0x90 - 0x9f
        (bcc::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcc", id: Rel}),
        (sta::<IndirectYAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (ahx::<IndirectYAddressing>, 6, false, OpcodeMarker{ name: "ahx", id: Iny}),
        (sty::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Zpx}),
        (sta::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Zpx}),
        (stx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Zpy}),
        (sax::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "sax", id: Zpy}),
        (tya::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tya", id: Imp}),
        (sta::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Aby}),
        (txs::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txs", id: Imp}),
        (tas::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "tas", id: Aby}),
        (shy::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "shy", id: Abx}),
        (sta::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Abx}),
        (shx::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "shx", id: Aby}),
        (ahx::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "ahx", id: Aby}),

        // 0xa0 - 0xaf
        (ldy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldy", id: Imm}),
        (lda::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "lda", id: Xin}),
        (ldx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldx", id: Imm}),
        (lax::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "lax", id: Xin}),
        (ldy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldy", id: Zpg}),
        (lda::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "lda", id: Zpg}),
        (ldx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldx", id: Zpg}),
        (lax::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "lax", id: Zpg}),
        (tay::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tay", id: Imp}),
        (lda::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lda", id: Imm}),
        (tax::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tax", id: Imp}),
        (lax::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lxa", id: Imm}),
        (ldy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Abs}),
        (lda::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Abs}),
        (ldx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Abs}),
        (lax::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Abs}),

        // 0xb0 - 0xbf
        (bcs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcs", id: Rel}),
        (lda::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lda", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (lax::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lax", id: Iny}),
        (ldy::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Zpx}),
        (lda::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Zpx}),
        (ldx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Zpy}),
        (lax::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "lax", id: Zpy}),
        (clv::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clv", id: Imp}),
        (lda::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Aby}),
        (tsx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tsx", id: Imp}),
        (las::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "las", id: Aby}),
        (ldy::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ldy", id: Abx}),
        (lda::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Abx}),
        (ldx::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ldx", id: Aby}),
        (lax::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lax", id: Aby}),

        // 0xc0 - 0xcf
        (cpy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpy", id: Imm}),
        (cmp::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "cmp", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (dcp::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "dcp", id: Xin}),
        (cpy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpy", id: Zpg}),
        (cmp::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cmp", id: Zpg}),
        (dec::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dec", id: Zpg}),
        (dcp::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dcp", id: Zpg}),
        (iny::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "iny", id: Imp}),
        (cmp::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cmp", id: Imm}),
        (dex::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dex", id: Imp}),
        (sbx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbx", id: Imm}),
        (cpy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpy", id: Abs}),
        (cmp::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Abs}),
        (dec::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Abs}),
        (dcp::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "dcp", id: Abs}),

        // 0xd0 - 0xdf
        (bne::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bne", id: Rel}),
        (cmp::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "cmp", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (dcp::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "dcp", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (cmp::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Zpx}),
        (dec::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Zpx}),
        (dcp::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "dcp", id: Zpx}),
        (cld::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cld", id: Imp}),
        (cmp::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (dcp::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "dcp", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (cmp::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Abx}),
        (dec::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "dec", id: Abx}),
        (dcp::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "dcp", id: Abx}),

        // 0xe0 - 0xef
        (cpx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpx", id: Imm}),
        (sbc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sbc", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (isc::<XIndirectAddressing>, 8, false, OpcodeMarker{ name: "isc", id: Xin}),
        (cpx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpx", id: Zpg}),
        (sbc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sbc", id: Zpg}),
        (inc::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "inc", id: Zpg}),
        (isc::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "isc", id: Zpg}),
        (inx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "inx", id: Imp}),
        (sbc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbc", id: Imm}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (sbc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbc", id: Imm}),
        (cpx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpx", id: Abs}),
        (sbc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Abs}),
        (inc::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Abs}),
        (isc::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "isc", id: Abs}),

        // 0xf0 - 0xff
        (beq::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "beq", id: Rel}),
        (sbc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "sbc", id: Iny}),
        (kil::<ImpliedAddressing>, 0, false, OpcodeMarker{ name: "kil", id: Imp}),
        (isc::<IndirectYAddressing>, 8, false, OpcodeMarker{ name: "isc", id: Iny}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (sbc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Zpx}),
        (inc::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Zpx}),
        (isc::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "isc", id: Zpx}),
        (sed::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sed", id: Imp}),
        (sbc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Aby}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (isc::<AbsoluteYAddressing>, 7, false, OpcodeMarker{ name: "isc", id: Aby}),
        (nop::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abx}),
        (sbc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Abx}),
        (inc::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "inc", id: Abx}),
        (isc::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "isc", id: Abx}),
    ];

/// 65C02 opcode table, same as above with the 65C02 differences.
pub(crate) static ref OPCODE_MATRIX_65C02: Vec<( fn(c: &mut Cpu, d: Option<&Debugger>, opcode_byte: u8, in_cycles: usize, extra_cycle_on_page_crossing: bool, decode_only: bool) -> Result<(i8, usize, Option<String>), CpuError>, usize, bool, OpcodeMarker)> =
    vec![
        // 0x0 - 0xf
        (brk::<ImpliedAddressing>, 7, false, OpcodeMarker{ name: "brk", id: Imp}),
        (ora::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "ora", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (tsb::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "tsb", id: Zpg}),
        (ora::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ora", id: Zpg}),
        (asl::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "asl", id: Zpg}),
        (rmb0::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb0", id: Zpg}),
        (php::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "php", id: Imp}),
        (ora::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ora", id: Imm}),
        (asl::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "asl", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (tsb::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "tsb", id: Abs}),
        (ora::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Abs}),
        (asl::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Abs}),
        (bbr0::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr0", id: Zpr}),

        // 0x10 - 0x1f
        (bpl::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bpl", id: Rel}),
        (ora::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "ora", id: Iny}),
        (ora::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "ora", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (trb::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "trb", id: Zpg}),
        (ora::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ora", id: Zpx}),
        (asl::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "asl", id: Zpx}),
        (rmb1::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb1", id: Zpg}),
        (clc::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clc", id: Imp}),
        (ora::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Aby}),
        (inc::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "inc", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (trb::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "trb", id: Abs}),
        (ora::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ora", id: Abx}),
        (asl::<AbsoluteXAddressing>, 6, true, OpcodeMarker{ name: "asl", id: Abx}),
        (bbr1::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr1", id: Zpr}),

        // 0x20 - 0x2f
        (jsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "jsr", id: Abs}),
        (and::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "and", id: Abx}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (bit::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "bit", id: Zpg}),
        (and::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "and", id: Zpg}),
        (rol::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rol", id: Zpg}),
        (rmb2::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb2", id: Zpg}),
        (plp::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "plp", id: Imp}),
        (and::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "and", id: Imm}),
        (rol::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "rol", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (bit::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "bit", id: Abs}),
        (and::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "and", id: Abs}),
        (rol::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Abs}),
        (bbr2::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr2", id: Zpr}),

        // 0x30 - 0x3f
        (bmi::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bmi", id: Rel}),
        (and::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "and", id: Iny}),
        (and::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "and", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (bit::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "bit", id: Zpx}),
        (and::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "and", id: Zpx}),
        (rol::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "rol", id: Zpx}),
        (rmb3::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb3", id: Zpg}),
        (sec::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sec", id: Imp}),
        (and::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "and", id: Aby}),
        (dec::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "dec", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (bit::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "bit", id: Abx}),
        (and::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "and", id: Abx}),
        (rol::<AbsoluteXAddressing>, 6, true, OpcodeMarker{ name: "rol", id: Abx}),
        (bbr3::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr3", id: Zpr}),

        // 0x40 - 0x4f
        (rti::<ImpliedAddressing>, 6, false, OpcodeMarker{ name: "rti", id: Imp}),
        (eor::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "eor", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "nop", id: Zpg}),
        (eor::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "eor", id: Zpg}),
        (lsr::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "lsr", id: Zpg}),
        (rmb4::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb4", id: Zpg}),
        (pha::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "pha", id: Imp}),
        (eor::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "eor", id: Imm}),
        (lsr::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "lsr", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (jmp::<AbsoluteAddressing>, 3, false, OpcodeMarker{ name: "jmp", id: Abs}),
        (eor::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Abs}),
        (lsr::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Abs}),
        (bbr4::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr4", id: Zpr}),

        // 0x50 - 0x5f
        (bvc::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bvc", id: Rel}),
        (eor::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "eor", id: Iny}),
        (eor::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "eor", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (eor::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "eor", id: Zpx}),
        (lsr::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "lsr", id: Zpx}),
        (rmb5::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb5", id: Zpg}),
        (cli::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cli", id: Imp}),
        (eor::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Aby}),
        (phy::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "phy", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<AbsoluteAddressing>, 8, false, OpcodeMarker{ name: "nop", id: Abs}),
        (eor::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "eor", id: Abx}),
        (lsr::<AbsoluteXAddressing>, 6, true, OpcodeMarker{ name: "lsr", id: Abx}),
        (bbr5::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr5", id: Zpr}),

        // 0x60 - 0x6f
        (rts::<ImpliedAddressing>, 6, false, OpcodeMarker{ name: "rts", id: Imp}),
        (adc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "adc", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (stz::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "stz", id: Zpg}),
        (adc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "adc", id: Zpg}),
        (ror::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "ror", id: Zpg}),
        (rmb6::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb6", id: Zpg}),
        (pla::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "pla", id: Imp}),
        (adc::<ImmediateAddressing>, 2, true, OpcodeMarker{ name: "adc", id: Imm}),
        (ror::<AccumulatorAddressing>, 2, false, OpcodeMarker{ name: "ror", id: Acc}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (jmp::<IndirectAddressing>, 6, false, OpcodeMarker{ name: "jmp", id: Ind}),
        (adc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Abs}),
        (ror::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Abs}),
        (bbr6::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr6", id: Zpr}),

        // 0x70 - 0x7f
        (bvs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bvs", id: Rel}),
        (adc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "adc", id: Iny}),
        (adc::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "adc", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (stz::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "stz", id: Zpx}),
        (adc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "adc", id: Zpx}),
        (ror::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "ror", id: Zpx}),
        (rmb7::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "rmb7", id: Zpg}),
        (sei::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sei", id: Imp}),
        (adc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Aby}),
        (ply::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "ply", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (jmp::<AbsoluteIndirectXAddressing>, 6, false, OpcodeMarker{ name: "jmp", id: Aix}),
        (adc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "adc", id: Abx}),
        (ror::<AbsoluteXAddressing>, 7, true, OpcodeMarker{ name: "ror", id: Abx}),
        (bbr7::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbr7", id: Zpr}),

        // 0x80 - 0x8f
        (bra::<RelativeAddressing>, 3, true, OpcodeMarker{ name: "bra", id: Rel}),
        (sta::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (sty::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sty", id: Zpg}),
        (sta::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sta", id: Zpg}),
        (stx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "stx", id: Zpg}),
        (smb0::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb0", id: Zpg}),
        (dey::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dey", id: Imp}),
        (bit::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "bit", id: Imm}),
        (txa::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txa", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (sty::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Abs}),
        (sta::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Abs}),
        (stx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Abs}),
        (bbs0::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs0", id: Zpr}),

        // 0x90 - 0x9f
        (bcc::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcc", id: Rel}),
        (sta::<IndirectYAddressing>, 6, false, OpcodeMarker{ name: "sta", id: Iny}),
        (sta::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "kil", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (sty::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sty", id: Zpx}),
        (sta::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sta", id: Zpx}),
        (stx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "stx", id: Zpy}),
        (smb1::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb1", id: Zpg}),
        (tya::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tya", id: Imp}),
        (sta::<AbsoluteYAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Aby}),
        (txs::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "txs", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (stz::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "stz", id: Abs}),
        (sta::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "sta", id: Abx}),
        (stz::<AbsoluteXAddressing>, 5, false, OpcodeMarker{ name: "stz", id: Abx}),
        (bbs1::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs1", id: Zpr}),

        // 0xa0 - 0xaf
        (ldy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldy", id: Imm}),
        (lda::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "lda", id: Xin}),
        (ldx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "ldx", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (ldy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldy", id: Zpg}),
        (lda::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "lda", id: Zpg}),
        (ldx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "ldx", id: Zpg}),
        (smb2::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb2", id: Zpg}),
        (tay::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tay", id: Imp}),
        (lda::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "lda", id: Imm}),
        (tax::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tax", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (ldy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Abs}),
        (lda::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Abs}),
        (ldx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Abs}),
        (bbs2::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs2", id: Zpr}),

        // 0xb0 - 0xbf
        (bcs::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bcs", id: Rel}),
        (lda::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "lda", id: Iny}),
        (lda::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "lda", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (ldy::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "ldy", id: Zpx}),
        (lda::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "lda", id: Zpx}),
        (ldx::<ZeroPageYAddressing>, 4, false, OpcodeMarker{ name: "ldx", id: Zpy}),
        (smb3::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb3", id: Zpg}),
        (clv::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "clv", id: Imp}),
        (lda::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Aby}),
        (tsx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "tsx", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (ldy::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "ldy", id: Abx}),
        (lda::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "lda", id: Abx}),
        (ldx::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "ldx", id: Aby}),
        (bbs3::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs3", id: Zpr}),

        // 0xc0 - 0xcf
        (cpy::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpy", id: Imm}),
        (cmp::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "cmp", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (cpy::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpy", id: Zpg}),
        (cmp::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cmp", id: Zpg}),
        (dec::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "dec", id: Zpg}),
        (smb4::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb4", id: Zpg}),
        (iny::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "iny", id: Imp}),
        (cmp::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cmp", id: Imm}),
        (dex::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "dex", id: Imp}),
        (wai::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "wai", id: Imp}),
        (cpy::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpy", id: Abs}),
        (cmp::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Abs}),
        (dec::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Abs}),
        (bbs4::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs4", id: Zpr}),

        // 0xd0 - 0xdf
        (bne::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "bne", id: Rel}),
        (cmp::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "cmp", id: Iny}),
        (cmp::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "cmp", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (cmp::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "cmp", id: Zpx}),
        (dec::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "dec", id: Zpx}),
        (smb5::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb5", id: Zpg}),
        (cld::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "cld", id: Imp}),
        (cmp::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Aby}),
        (phx::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "phx", id: Imp}),
        (stp::<ImpliedAddressing>, 3, false, OpcodeMarker{ name: "stp", id: Imp}),
        (nop::<AbsoluteAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abs}),
        (cmp::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "cmp", id: Abx}),
        (dec::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "dec", id: Abx}),
        (bbs5::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs5", id: Zpr}),

        // 0xe0 - 0xef
        (cpx::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "cpx", id: Imm}),
        (sbc::<XIndirectAddressing>, 6, false, OpcodeMarker{ name: "sbc", id: Xin}),
        (nop::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imm}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (cpx::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "cpx", id: Zpg}),
        (sbc::<ZeroPageAddressing>, 3, false, OpcodeMarker{ name: "sbc", id: Zpg}),
        (inc::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "inc", id: Zpg}),
        (smb6::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb6", id: Zpg}),
        (inx::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "inx", id: Imp}),
        (sbc::<ImmediateAddressing>, 2, false, OpcodeMarker{ name: "sbc", id: Imm}),
        (nop::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (cpx::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "cpx", id: Abs}),
        (sbc::<AbsoluteAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Abs}),
        (inc::<AbsoluteAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Abs}),
        (bbs6::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs6", id: Zpr}),

        // 0xf0 - 0xff
        (beq::<RelativeAddressing>, 2, true, OpcodeMarker{ name: "beq", id: Rel}),
        (sbc::<IndirectYAddressing>, 5, true, OpcodeMarker{ name: "sbc", id: Iny}),
        (sbc::<IndirectZeroPageAddressing>, 5, false, OpcodeMarker{ name: "sbc", id: Izp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "nop", id: Zpx}),
        (sbc::<ZeroPageXAddressing>, 4, false, OpcodeMarker{ name: "sbc", id: Zpx}),
        (inc::<ZeroPageXAddressing>, 6, false, OpcodeMarker{ name: "inc", id: Zpx}),
        (smb7::<ZeroPageAddressing>, 5, false, OpcodeMarker{ name: "smb7", id: Zpg}),
        (sed::<ImpliedAddressing>, 2, false, OpcodeMarker{ name: "sed", id: Imp}),
        (sbc::<AbsoluteYAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Aby}),
        (plx::<ImpliedAddressing>, 4, false, OpcodeMarker{ name: "plx", id: Imp}),
        (nop::<ImpliedAddressing>, 1, false, OpcodeMarker{ name: "nop", id: Imp}),
        (nop::<AbsoluteAddressing>, 4, true, OpcodeMarker{ name: "nop", id: Abs}),
        (sbc::<AbsoluteXAddressing>, 4, true, OpcodeMarker{ name: "sbc", id: Abx}),
        (inc::<AbsoluteXAddressing>, 7, false, OpcodeMarker{ name: "inc", id: Abx}),
        (bbs7::<ZeroPageRelativeAddressing>, 5, false, OpcodeMarker{ name: "bbs7", id: Zpr}),
    ];
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
pub(super) fn push_byte(c: &mut Cpu, d: Option<&Debugger>, b: u8) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    let addr = 0x100 + c.regs.s as usize;
    mem.write_byte(addr, b)?;
    c.regs.s = c.regs.s.wrapping_sub(1);
    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::WRITE)?
    }

    // call callback if any
    c.call_callback(addr as u16, b, 1, CpuOperation::Write);
    Ok(())
}

/**
 * pop byte off the stack
 */
fn pop_byte(c: &mut Cpu, d: Option<&Debugger>) -> Result<u8, CpuError> {
    let mem = c.bus.get_memory();
    c.regs.s = c.regs.s.wrapping_add(1);
    let addr = 0x100 + c.regs.s as usize;
    let b = mem.read_byte(addr)?;

    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::READ)?
    }

    // call callback if any
    c.call_callback(addr as u16, b, 1, CpuOperation::Read);
    Ok(b)
}

/**
 * pop word off the stack
 */
fn pop_word_le(c: &mut Cpu, d: Option<&Debugger>) -> Result<u16, CpuError> {
    let mem = c.bus.get_memory();
    c.regs.s = c.regs.s.wrapping_add(2);
    let addr = 0x100 + (c.regs.s - 1) as usize;

    let w = mem.read_word_le(addr)?;

    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::READ)?
    }

    // call callback if any
    c.call_callback(addr as u16, (w & 0xff) as u8, 2, CpuOperation::Read);

    Ok(w)
}

/**
 * push word on the stack
 */
pub(super) fn push_word_le(c: &mut Cpu, d: Option<&Debugger>, w: u16) -> Result<(), CpuError> {
    let mem = c.bus.get_memory();
    let addr = 0x100 + (c.regs.s - 1) as usize;
    mem.write_word_le(addr, w)?;
    c.regs.s = c.regs.s.wrapping_sub(2);

    // handle breakpoint
    if d.is_some() {
        d.unwrap()
            .handle_rw_breakpoint(c, addr as u16, BreakpointType::WRITE)?
    }

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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    let mut cycles = in_cycles;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((A::len(), cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

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
    A::store(c, d, tgt, res)?;

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // and (preserve flags, n and z are set in lfr)
    let prev_p = c.regs.p;
    and::<A>(
        c,
        d,
        opcode_byte,
        0,
        _extra_cycle_on_page_crossing,
        decode_only,
    )?;
    c.regs.p = prev_p;

    // lsr A
    lsr::<AccumulatorAddressing>(c, d, opcode_byte, 0, false, decode_only)?;

    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // and
    and::<A>(
        c,
        d,
        opcode_byte,
        in_cycles,
        _extra_cycle_on_page_crossing,
        decode_only,
    )?;
    c.set_cpu_flags(CpuFlags::C, utils::is_signed(c.regs.a));
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

    // A AND M -> A
    c.regs.a = c.regs.a & b;

    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    if !c.is_cpu_flag_set(CpuFlags::D) {
        // and
        and::<A>(c, d, opcode_byte, 0, false, decode_only)?;

        // ror A
        let prev_a = c.regs.a;
        ror::<AccumulatorAddressing>(c, d, opcode_byte, 0, false, decode_only)?;

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
        and::<A>(c, d, opcode_byte, 0, false, decode_only)?;
        let and_res = c.regs.a;
        ror::<AccumulatorAddressing>(c, d, opcode_byte, 0, false, decode_only)?;

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
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;
    c.set_cpu_flags(CpuFlags::C, utils::is_signed(b));

    // shl
    b <<= 1;
    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    let and_res = c.regs.a & b;

    c.set_cpu_flags(CpuFlags::Z, and_res == 0);

    // on 65c02 and immediate mode, N and V are not affected
    if c.cpu_type == CpuType::MOS6502
        || (c.cpu_type == CpuType::WDC65C02 && A::id() != AddressingModeId::Imm)
    {
        c.set_cpu_flags(CpuFlags::N, utils::is_signed(b));
        c.set_cpu_flags(CpuFlags::V, b & 0b01000000 != 0);
    }
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    let mut cycles = in_cycles;
    let mut taken: bool = false;
    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    let mut cycles = in_cycles;
    let mut taken: bool = false;
    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // push pc and p on stack
    push_word_le(c, d, c.regs.pc + 2)?;

    // push P with U and B set
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    let mut flags = c.regs.p.clone();
    flags.set(CpuFlags::B, true);
    flags.set(CpuFlags::U, true);
    push_byte(c, d, flags.bits())?;

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
    Ok((0, in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        cycles + if extra_cycle { 1 } else { 0 },
        None,
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // clear carry
    c.set_cpu_flags(CpuFlags::C, false);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // clear decimal flag
    c.set_cpu_flags(CpuFlags::D, false);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    // enable interrupts, clear the flag
    c.set_cpu_flags(CpuFlags::I, false);

    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    // clear the overflow flag
    c.set_cpu_flags(CpuFlags::V, false);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    let res = c.regs.a.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.a >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.a == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    let res = c.regs.x.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.x >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.x == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    let res = c.regs.y.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.y >= b);
    c.set_cpu_flags(CpuFlags::Z, c.regs.y == b);
    c.set_cpu_flags(CpuFlags::N, utils::is_signed(res));

    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // perform dec + cmp internally (flags are set according to cmp, so save before)
    let prev_p = c.regs.p;
    dec::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    c.regs.p = prev_p;
    cmp::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;
    b = b.wrapping_sub(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.x = c.regs.x.wrapping_sub(1);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.y = c.regs.y.wrapping_sub(1);
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    c.regs.a = c.regs.a ^ b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;

    b = b.wrapping_add(1);
    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.x = c.regs.x.wrapping_add(1);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.y = c.regs.y.wrapping_add(1);
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    // perform inc + sbc internally (sbc sets p, preserve carry flag after inc)
    let prev_p = c.regs.p;
    inc::<A>(c, d, opcode_byte, 0, false, decode_only)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    sbc::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
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

    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // push return address
    push_word_le(
        c,
        d,
        c.regs.pc.wrapping_add(A::len() as u16).wrapping_sub(1),
    )?;

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

    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }, None))
}

/**
 * CPU JAM!
 */
#[named]
fn kil<A: AddressingMode>(
    c: &mut Cpu,
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    _in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    // this is an invalid opcode and emulation should be halted!
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // get operand
    let b = A::load(c, d, tgt)?;
    let res = b & c.regs.s;

    c.regs.a = res;
    c.regs.x = res;
    c.regs.s = res;
    set_zn_flags(c, res);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    c.regs.a = b;
    c.regs.x = b;
    set_zn_flags(c, b);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;
    c.regs.a = b;

    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;
    c.regs.x = b;

    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;
    c.regs.y = b;

    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;

    // save bit 0 in the carry
    c.set_cpu_flags(CpuFlags::C, b & 1 != 0);

    // lsr
    b >>= 1;

    set_zn_flags(c, b);

    // store back
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    // N and Z are set according to the value of the accumulator before the instruction executed
    set_zn_flags(c, c.regs.a);

    // we choose $ee as constant as specified in [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    let k = 0xee;
    let res: u8 = (c.regs.a | k) & b;
    c.regs.x = res;
    c.regs.a = res;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // noop, do nothing ...
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;
    c.regs.a |= b;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    push_byte(c, d, c.regs.a)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    // ensure B and U(ndefined) are set to 1
    let mut flags = c.regs.p.clone();
    flags.set(CpuFlags::U, true);
    flags.set(CpuFlags::B, true);
    push_byte(c, d, flags.bits())?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.a = pop_byte(c, d)?;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let popped_flags = pop_byte(c, d)?;
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
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // perform rol + and internally
    let prev_p = c.regs.p;
    rol::<A>(c, d, opcode_byte, 0, false, decode_only)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);
    // n and z are set according to AND
    and::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;

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
    A::store(c, d, tgt, b)?;
    set_zn_flags(c, b);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;

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
    A::store(c, d, tgt, b)?;
    set_zn_flags(c, b);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // perform ror + adc internally
    let prev_p = c.regs.p;
    ror::<A>(c, d, opcode_byte, 0, false, decode_only)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // all other flags are set by adc
    adc::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let popped_flags = pop_byte(c, d)?;
    c.regs.p = CpuFlags::from_bits(popped_flags).unwrap();

    // ensure flag Unused is set and B is unset
    c.set_cpu_flags(CpuFlags::B, false);
    c.set_cpu_flags(CpuFlags::U, true);

    // pull pc
    c.regs.pc = pop_word_le(c, d)?;

    // apply fix if needed, and anyway reset the flag.
    //c.regs.pc = c.regs.pc.wrapping_add(c.fix_pc_rti as u16);
    //c.fix_pc_rti = 0;
    c.processing_ints = false;
    println!("returning from RTI at pc=${:04x}", c.regs.pc);
    Ok((0, in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.pc = pop_word_le(c, d)?.wrapping_add(1);
    Ok((0, in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    let b = c.regs.a & c.regs.x;
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // get target_address
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    let mut cycles = in_cycles;

    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((A::len(), cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    // and
    let and_res = c.regs.a & c.regs.x;

    // cmp
    c.regs.x = and_res.wrapping_sub(b);
    c.set_cpu_flags(CpuFlags::C, c.regs.a >= b);
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // set carry
    c.set_cpu_flags(CpuFlags::C, true);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    // set decimal flag
    c.set_cpu_flags(CpuFlags::D, true);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    // disable interrupts
    c.set_cpu_flags(CpuFlags::I, true);
    Ok((A::len(), in_cycles, None))
}

/**
 * SHX (undoc) (aka A11, SXA, XAS)
 *
 * Stores X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is dropped, page boundary crossings may not work
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
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
    A::store(c, d, tgt, res)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
}

/**
 * SHY (undoc) (aka A11, SYA, SAY)
 *
 * Stores Y AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is dropped, page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
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
    A::store(c, d, tgt, res)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // perform asl + ora internally
    let prev_p = c.regs.p;
    asl::<A>(c, d, opcode_byte, 0, false, decode_only)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // other flags are set by ora
    ora::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // perform lsr + eor internally
    let prev_p = c.regs.p;
    lsr::<A>(c, d, opcode_byte, 0, false, decode_only)?;

    // preserve carry
    let is_c_set = c.is_cpu_flag_set(CpuFlags::C);
    c.regs.p = prev_p;
    c.set_cpu_flags(CpuFlags::C, is_c_set);

    // other flags are set by eor
    eor::<A>(c, d, opcode_byte, 0, false, decode_only)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // store A in memory
    A::store(c, d, tgt, c.regs.a)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // store X in memory
    A::store(c, d, tgt, c.regs.x)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // store y in memory
    A::store(c, d, tgt, c.regs.y)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
}

/**
 * TAS (undoc) (aka XAS, SHS)
 *
 * Puts A AND X in SP and stores A AND X AND (high-byte of addr. + 1) at addr.
 *
 * unstable: sometimes 'AND (H+1)' is dropped, page boundary crossings may not work (with the high-byte of the value used as the high-byte of the address)
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
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
    A::store(c, d, tgt, res)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.x = c.regs.a;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.y = c.regs.a;
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.x = c.regs.s;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    c.regs.a = c.regs.x;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.s = c.regs.x;
    Ok((A::len(), in_cycles, None))
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
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.a = c.regs.y;
    set_zn_flags(c, c.regs.a);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

    // N and Z are set according to the value of the accumulator before the instruction executed
    set_zn_flags(c, c.regs.a);

    // we choose $ef as constant as specified in [https://csdb.dk/release/?id=198357](NMOS 6510 Unintended Opcodes)
    let k = 0xef;
    let res: u8 = (c.regs.a | k) & c.regs.x & b;
    c.regs.a = res;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
}

/**
 * 65c02 only opcodes following
 */

fn bbr_bbs_internal<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    bit: i8,
    name: &str,
    is_bbr: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, name)?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let b = A::load(c, d, tgt)?;

    // get byte to test
    let to_test_addr = A::load(c, d, c.regs.pc.wrapping_add(1))?;
    let to_test = A::load(c, d, to_test_addr as u16)?;

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
    Ok((
        if taken { 0 } else { A::len() },
        in_cycles + if extra_cycle { 1 } else { 0 },
        None,
    ))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        0,
        function_name!(),
        true,
    )
}

#[named]
fn bbr1<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        1,
        function_name!(),
        true,
    )
}

#[named]
fn bbr2<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        2,
        function_name!(),
        true,
    )
}

#[named]
fn bbr3<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        3,
        function_name!(),
        true,
    )
}

#[named]
fn bbr4<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        4,
        function_name!(),
        true,
    )
}

#[named]
fn bbr5<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        5,
        "bbr5",
        true,
    )
}

#[named]
fn bbr6<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        6,
        function_name!(),
        true,
    )
}

#[named]
fn bbr7<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        7,
        function_name!(),
        true,
    )
}

#[named]
fn bbs0<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        0,
        function_name!(),
        false,
    )
}

#[named]
fn bbs1<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        1,
        function_name!(),
        false,
    )
}

#[named]
fn bbs2<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        2,
        function_name!(),
        false,
    )
}

#[named]
fn bbs3<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        3,
        function_name!(),
        false,
    )
}

#[named]
fn bbs4<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        4,
        function_name!(),
        false,
    )
}

#[named]
fn bbs5<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        5,
        function_name!(),
        false,
    )
}

#[named]
fn bbs6<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        6,
        function_name!(),
        false,
    )
}

#[named]
fn bbs7<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    bbr_bbs_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        7,
        function_name!(),
        false,
    )
}

fn rmb_smb_internal<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
    bit: i8,
    name: &str,
    is_rmb: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, name)?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let mut b = A::load(c, d, tgt)?;

    if is_rmb {
        // reset bit
        b &= !(1 << bit);
    } else {
        // set bit
        b |= 1 << bit;
    }

    // write
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        0,
        function_name!(),
        true,
    )
}

#[named]
fn rmb1<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        1,
        function_name!(),
        true,
    )
}

#[named]
fn rmb2<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        2,
        function_name!(),
        true,
    )
}

#[named]
fn rmb3<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        3,
        function_name!(),
        true,
    )
}

#[named]
fn rmb4<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        4,
        function_name!(),
        true,
    )
}

#[named]
fn rmb5<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        5,
        function_name!(),
        true,
    )
}

#[named]
fn rmb6<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        6,
        function_name!(),
        true,
    )
}

#[named]
fn rmb7<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        7,
        function_name!(),
        true,
    )
}

#[named]
fn smb0<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        0,
        function_name!(),
        false,
    )
}

#[named]
fn smb1<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        1,
        function_name!(),
        false,
    )
}

#[named]
fn smb2<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        2,
        function_name!(),
        false,
    )
}

#[named]
fn smb3<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        3,
        function_name!(),
        false,
    )
}

#[named]
fn smb4<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        4,
        function_name!(),
        false,
    )
}

#[named]
fn smb5<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        5,
        function_name!(),
        false,
    )
}

#[named]
fn smb6<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
        6,
        function_name!(),
        false,
    )
}

#[named]
fn smb7<A: AddressingMode>(
    c: &mut Cpu,
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    rmb_smb_internal::<A>(
        c,
        d,
        _opcode_byte,
        in_cycles,
        extra_cycle_on_page_crossing,
        decode_only,
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let b = A::load(c, d, tgt)?;

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
    Ok((0, in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    push_byte(c, d, c.regs.x)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    push_byte(c, d, c.regs.y)?;
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.x = pop_byte(c, d)?;
    set_zn_flags(c, c.regs.x);
    Ok((A::len(), in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    c.regs.y = pop_byte(c, d)?;
    set_zn_flags(c, c.regs.y);
    Ok((A::len(), in_cycles, None))
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
    c: &mut Cpu,
    _: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    // deadlock
    Ok((0, in_cycles, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // store
    A::store(c, d, tgt, 0)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;
    // read operand
    let mut b = A::load(c, d, tgt)?;

    let res = (b & c.regs.a) == 0;
    c.set_cpu_flags(CpuFlags::Z, res);
    b &= !(c.regs.a);
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
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
    d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }
    let (tgt, extra_cycle) = A::target_address(c, extra_cycle_on_page_crossing)?;

    // read operand
    let mut b = A::load(c, d, tgt)?;
    let res = (b & c.regs.a) == 0;
    c.set_cpu_flags(CpuFlags::Z, res);
    b |= c.regs.a;
    A::store(c, d, tgt, b)?;
    Ok((A::len(), in_cycles + if extra_cycle { 1 } else { 0 }, None))
}

#[named]
fn wai<A: AddressingMode>(
    c: &mut Cpu,
    _d: Option<&Debugger>,
    _opcode_byte: u8,
    in_cycles: usize,
    _extra_cycle_on_page_crossing: bool,
    decode_only: bool,
) -> Result<(i8, usize, Option<String>), CpuError> {
    if decode_only {
        // just decode
        let opc_string = A::repr(c, function_name!())?;
        return Ok((A::len(), 0, Some(opc_string)));
    }

    let mut len = A::len();
    if !c.must_trigger_irq && !c.must_trigger_nmi {
        // will wait for interrupt
        len = 0;
    }
    Ok((len, in_cycles, None))
}
