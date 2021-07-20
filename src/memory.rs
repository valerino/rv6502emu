/*
 * Filename: /src/memory.rs
 * Project: rv6502emu
 * Created Date: Saturday, July 3rd 2021, 10:44:18 am
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

mod mem_error;
use mem_error::MemoryOperation;
use std::error::Error;

/**
 * trait for the emulated memory exposed by the cpu.
 *
 */
pub trait Memory {
    /**
     * reads a byte at address
     */
    fn read_byte(&self, address: usize) -> Result<u8, mem_error::MemoryError>;

    /**
     * reads a word (little-endian) at address
     */
    fn read_word_le(&self, address: usize) -> Result<u16, mem_error::MemoryError>;

    /**
     * writes a byte at address
     */
    fn write_byte(&mut self, address: usize, b: u8) -> Result<bool, mem_error::MemoryError>;

    /**
     * get memory size
     */
    fn size(&self) -> usize;

    /**
     * load file at address
     */
    fn load(&self, path: &str) -> Result<bool, mem_error::MemoryError>;
}

/**
 * implements the Memory trait.
 *
 * > *(default implementation)*
 */
pub(crate) struct EmulatedMemory {
    size: usize,
    m: Vec<u8>,
}

impl Memory for EmulatedMemory {
    fn read_byte(&self, address: usize) -> Result<u8, mem_error::MemoryError> {
        mem_error::check_address(self, address, 1, MemoryOperation::Read)?;
        Ok(self.m[address])
    }

    fn read_word_le(&self, address: usize) -> Result<u16, mem_error::MemoryError> {
        mem_error::check_address(self, address, 2, MemoryOperation::Read)?;
        let h: u16 = self.m[address].into();
        let l: u16 = self.m[address + 1].into();
        Ok((h << 8) | l)
    }

    fn write_byte(&mut self, address: usize, b: u8) -> Result<bool, mem_error::MemoryError> {
        mem_error::check_address(self, address, 1, MemoryOperation::Write)?;
        self.m[address] = b;
        Ok(false)
    }

    fn size(&self) -> usize {
        self.size
    }

    fn load(&self, path: &str) -> Result<bool, mem_error::MemoryError> {
        Ok(true)
    }
}

/**
 * returns an istance of Memory with the given size
 */
pub fn new(size: usize) -> impl Memory {
    // create memory and zero it
    let mut v = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(0)
    }
    EmulatedMemory { size: size, m: v }
}
