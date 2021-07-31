/*
 * Filename: /src/memory/mem_error.rs
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

use super::Memory;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum MemoryOperation {
    Read,
    Write,
    Load,
}

impl std::fmt::Display for MemoryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryOperation::Read => write!(f, "Read"),
            MemoryOperation::Write => write!(f, "Write"),
            MemoryOperation::Load => write!(f, "Load"),
        }
    }
}

#[derive(Debug)]
pub struct MemoryError {
    operation: MemoryOperation,
    address: usize,
    access_size: usize,
    mem_size: usize,
    msg: String,
}

impl std::error::Error for MemoryError {}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        if self.operation == MemoryOperation::Load {
            write!(f, "MemoryError ({}), msg={}", self.operation, self.msg,)
        } else {
            write!(
                f,
                "MemoryError ({}) at address=${:x}, access size={}, max memory size={}",
                self.operation, self.address, self.access_size, self.mem_size,
            )
        }
    }
}

impl From<std::io::Error> for MemoryError {
    fn from(err: std::io::Error) -> Self {
        MemoryError {
            operation: MemoryOperation::Load,
            address: 0,
            access_size: 0,
            mem_size: 0,
            msg: err.to_string(),
        }
    }
}

/**
 * check memory boundaries
 */
pub fn check_address(
    mem: &dyn Memory,
    address: usize,
    access_size: usize,
    op: MemoryOperation,
) -> Result<(), MemoryError> {
    // check if memory access overflows
    let mem_size = mem.get_size();
    if address + access_size > mem_size {
        // overflow
        let e = MemoryError {
            operation: op,
            address: address,
            mem_size: mem_size,
            access_size: access_size,
            msg: String::from(""),
        };
        return Err(e);
    }
    Ok(())
}
