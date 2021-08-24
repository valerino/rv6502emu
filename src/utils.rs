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
 * returns 1 if string is prepended with $, 0 otherwise.
 */
pub(crate) fn is_dollar_hex(v: &str) -> usize {
    if v.chars().next().unwrap_or_default() != '$' {
        return 0;
    }
    return 1;
}

/**
 * activate logging on stdout through env_logger (max level).
 */
pub(crate) fn enable_logging_internal(enable: bool) {
    if enable == true {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .try_init();
        log::set_max_level(log::LevelFilter::max());
    } else {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Off)
            .try_init();
        log::set_max_level(log::LevelFilter::Off);
    }
}

/**
 * check if log is enabled
 */
pub(crate) fn log_enabled() -> bool {
    log::max_level() == Level::max()
}

/**
 * display opcode string, currently implemented to stdout
 */
pub(crate) fn debug_out_opcode<A: AddressingMode>(
    c: &mut Cpu,
    opcode_name: &str,
) -> Result<(), CpuError> {
    if log_enabled() {
        let opc_string = A::repr(c, opcode_name)?;
        println!("\t{}", opc_string);
    }
    Ok(())
}

/**
 * display registers and cycles, currently implemented to stdout
 */
pub(crate) fn debug_out_registers(c: &Cpu) {
    println!("\t{}, cycles={}", c.regs, c.cycles);
}
