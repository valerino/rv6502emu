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
pub enum ErrorType {
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
}
pub type CpuErrorType = self::ErrorType;

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::MemoryRead => write!(f, "MemRead"),
            ErrorType::MemoryWrite => write!(f, "MemWrite"),
            ErrorType::MemoryLoad => write!(f, "MemLoad"),
            ErrorType::InvalidOpcode => write!(f, "InvalidOpcode"),
            ErrorType::RwBreakpoint => write!(f, "RW Breakpoint"),
        }
    }
}

/**
 * to report errors within the whole crate
 */
#[derive(Debug)]
pub struct Error {
    /// one of the defined ErrorType enums.
    pub operation: ErrorType,
    /// error address.
    pub address: usize,
    /// read/write requested access size which caused the error.\
    pub access_size: usize,
    /// whole memory size.
    pub mem_size: usize,
    /// the breakpoint index which triggered, if operation is RwBreakpoint
    pub bp_idx: i8,
    /// an optional message.
    pub msg: Option<String>,
}
pub type CpuError = self::Error;

impl std::error::Error for CpuError {}

impl std::fmt::Display for CpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        if self.operation == ErrorType::MemoryLoad {
            write!(
                f,
                "Error ({}), msg={}",
                self.operation,
                self.msg.as_ref().unwrap(),
            )
        } else if self.operation == ErrorType::InvalidOpcode {
            write!(f, "Error ({})", self.operation,)
        } else if self.operation == ErrorType::RwBreakpoint {
            write!(f, "Error ({}), bp index={}", self.operation, self.bp_idx)
        } else {
            write!(
                f,
                "Error ({}) at address=${:x}, access size={}, max memory size=${:04x} ({})",
                self.operation, self.address, self.access_size, self.mem_size, self.mem_size,
            )
        }
    }
}

impl From<std::io::Error> for CpuError {
    fn from(err: std::io::Error) -> Self {
        let e = CpuError {
            operation: ErrorType::MemoryLoad,
            address: 0,
            mem_size: 0,
            access_size: 0,
            bp_idx: 0,
            msg: Some(err.to_string()),
        };
        e
    }
}

/**
 * creates an invalid opcode error.
 */
pub(crate) fn new_invalid_opcode_error(address: usize) -> CpuError {
    let e = CpuError {
        operation: ErrorType::InvalidOpcode,
        address: address,
        mem_size: 0,
        access_size: 0,
        bp_idx: 0,
        msg: None,
    };
    return e;
}

/**
 * check memory boundaries during access
 */
pub(crate) fn check_address_boundaries(
    mem_size: usize,
    address: usize,
    access_size: usize,
    // we use the ErrorType to identify the operation (read/write/load)
    op: ErrorType,
    msg: Option<String>,
) -> Result<(), Error> {
    // check if memory access overflows
    if (address + access_size - 1 > mem_size) || (address + access_size - 1) > 0xffff {
        // report read or write error
        let e = CpuError {
            operation: op,
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
    op: ErrorType,
    msg: Option<String>,
) -> Result<(), Error> {
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
