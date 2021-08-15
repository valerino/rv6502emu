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

use crate::memory::Memory;
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
}
pub type CpuErrorType = self::ErrorType;

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::MemoryRead => write!(f, "MemRead"),
            ErrorType::MemoryWrite => write!(f, "MemWrite"),
            ErrorType::MemoryLoad => write!(f, "MemLoad"),
            ErrorType::InvalidOpcode => write!(f, "InvalidOpcode"),
        }
    }
}

/**
 * to report errors within the whole crate
 */
#[derive(Debug)]
pub struct Error {
    pub operation: ErrorType,
    address: usize,
    access_size: usize,
    mem_size: usize,
    msg: Option<String>,
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
        } else {
            write!(
                f,
                "Error ({}) at address=${:x}, access size={}, max memory size={}",
                self.operation, self.address, self.access_size, self.mem_size,
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
            msg: msg,
        };
        return Err(e);
    }
    Ok(())
}
