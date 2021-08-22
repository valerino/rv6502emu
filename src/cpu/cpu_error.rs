/*
 * Filename: /src/cpu/cpu_error.rs
 * Project: rv6502emu
 * Created Date: Saturday, July 5rd 2021, 09:11:09 am
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

use crate::cpu::addressing_modes::AddressingModeId;
use std::fmt;

/**
 * type of cpu error.
 */
#[derive(PartialEq, Debug)]
pub enum CpuErrorType {
    /// reads from memory.
    MemoryRead,
    /// writes to memory.
    MemoryWrite,
    /// loads a file to memory.
    MemoryLoad,
    /// invalid instruction.
    InvalidOpcode,
    /// read/write breakpoint hit.
    RwBreakpoint,
    /// generic error
    Generic,
}

impl std::fmt::Display for CpuErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuErrorType::MemoryRead => write!(f, "MemRead"),
            CpuErrorType::MemoryWrite => write!(f, "MemWrite"),
            CpuErrorType::MemoryLoad => write!(f, "MemLoad"),
            CpuErrorType::InvalidOpcode => write!(f, "InvalidOpcode"),
            CpuErrorType::RwBreakpoint => write!(f, "RwBreakpoint"),
            CpuErrorType::Generic => write!(f, "Generic"),
        }
    }
}

/**
 * to report errors within the whole crate
 */
#[derive(Debug)]
pub struct CpuError {
    /// one of the defined ErrorType enums.
    pub t: CpuErrorType,
    /// error address.
    pub address: usize,
    /// read/write requested access size which caused the error.
    pub access_size: usize,
    /// whole memory size.
    pub mem_size: usize,
    /// the breakpoint index which triggered, if t is RwBreakpoint.
    pub bp_idx: i8,
    /// an optional message.
    pub msg: Option<String>,
}

impl std::error::Error for CpuError {}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self.t {
            CpuErrorType::MemoryLoad => {
                write!(f, "Error ({}), msg={}", self.t, self.msg.as_ref().unwrap(),)
            }
            CpuErrorType::InvalidOpcode => {
                write!(f, "Error ({})", self.t,)
            }
            CpuErrorType::Generic => {
                write!(
                    f,
                    "Error ({}) {}",
                    self.t,
                    self.msg.as_ref().unwrap_or(&String::from(""))
                )
            }
            CpuErrorType::RwBreakpoint => {
                write!(f, "Error ({}), bp index={}", self.t, self.bp_idx)
            }
            _ => {
                write!(
                    f,
                    "Error ({}) at address=${:x}, access size={}, max memory size=${:04x} ({})",
                    self.t, self.address, self.access_size, self.mem_size, self.mem_size,
                )
            }
        }
    }
}

impl From<std::io::Error> for CpuError {
    fn from(err: std::io::Error) -> Self {
        let e = CpuError {
            t: CpuErrorType::MemoryLoad,
            address: 0,
            mem_size: 0,
            access_size: 0,
            bp_idx: 0,
            msg: Some(err.to_string()),
        };
        e
    }
}
impl CpuError {
    /**
     * constructs a new default error, with optional message
     */
    pub fn new_default(t: CpuErrorType, m: Option<String>) -> Self {
        let e = CpuError {
            t: t,
            address: 0,
            mem_size: 0,
            access_size: 0,
            bp_idx: 0,
            msg: m,
        };
        e
    }
}

/**
 * check memory boundaries during access
 */
pub(crate) fn check_address_boundaries(
    mem_size: usize,
    address: usize,
    access_size: usize,
    // we use the ErrorType to identify the operation (read/write/load)
    op: CpuErrorType,
    msg: Option<String>,
) -> Result<(), CpuError> {
    // check if memory access overflows
    if (address + access_size - 1 > mem_size) || (address + access_size - 1) > 0xffff {
        // report read or write error
        let e = CpuError {
            t: op,
            address: address,
            mem_size: mem_size,
            access_size: access_size,
            bp_idx: 0,
            msg: msg,
        };
        return Err(e);
    }
    Ok(())
}

/**
 * check memory boundaries during opcode access
 */
pub(crate) fn check_opcode_boundaries(
    mem_size: usize,
    address: usize,
    addr_mode: AddressingModeId,
    op: CpuErrorType,
    msg: Option<String>,
) -> Result<(), CpuError> {
    match addr_mode {
        AddressingModeId::Imp | AddressingModeId::Acc => {
            check_address_boundaries(mem_size, address, 1, op, msg)?;
        }
        AddressingModeId::Abs
        | AddressingModeId::Abx
        | AddressingModeId::Aby
        | AddressingModeId::Ind => {
            check_address_boundaries(mem_size, address, 3, op, msg)?;
        }
        AddressingModeId::Rel
        | AddressingModeId::Imm
        | AddressingModeId::Zpg
        | AddressingModeId::Zpx
        | AddressingModeId::Zpy
        | AddressingModeId::Iny
        | AddressingModeId::Xin => {
            check_address_boundaries(mem_size, address, 2, op, msg)?;
        }
    }
    Ok(())
}
