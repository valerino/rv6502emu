/*
 * Filename: /src/utils.rs
 * Project: rv6502emu
 * Created Date: 2021-08-11, 09:16:30
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

use crate::cpu::addressing_modes::AddressingMode;
use crate::cpu::cpu_error::CpuError;
use crate::cpu::Cpu;
use log::*;
use std::fmt::Display;

/**
 * simply check bit 7 for signed/unsigned byte
 */
pub(crate) fn is_signed(n: u8) -> bool {
    if (n & 0x80) == 0 {
        return false;
    }
    true
}

/**
 * display opcode string, currently implemented to stdout
 */
pub(crate) fn debug_out_opcode<A: AddressingMode>(
    c: &mut Cpu,
    opcode_name: &str,
) -> Result<(), CpuError> {
    if log::log_enabled!(Level::max()) {
        let opc_string = A::repr(c, opcode_name)?;
        //debug!("\t{}", opc_string);
        println!("\t{}", opc_string);
    }
    Ok(())
}

/**
 * returns 1 if string is prepended with $, 0 otherwise.
 */
pub(crate) fn is_dollar_hex(v: &str) -> usize {
    if v.chars().next().unwrap_or_default() != '$' {
        return 0;
    }
    return 1;
}

/**
 * display opcode string, currently implemented to stdout
 */
pub(crate) fn debug_out_text(d: &dyn Display) {
    if log::log_enabled!(Level::max()) {
        //debug!("{}", s);
        println!("{}", d);
    }
}

/**
 * display registers and cycles, currently implemented to stdout
 */
pub(crate) fn debug_out_registers(c: &Cpu) {
    if log::log_enabled!(Level::max()) {
        //debug!("{}", self.regs);
        println!("\t{}, cycles={}", c.regs, c.cycles);
    }
}
