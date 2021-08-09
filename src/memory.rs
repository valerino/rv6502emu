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

pub mod memory_error;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use memory_error::{MemoryError, MemoryOperation};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;

/**
 * trait for the emulated memory exposed by the cpu.
 *
 */
pub trait Memory {
    /**
     * reads a byte at address.
     */
    fn read_byte(&mut self, address: usize) -> Result<u8, MemoryError>;

    /**
     * reads a word (little-endian) at address.
     */
    fn read_word_le(&mut self, address: usize) -> Result<u16, MemoryError>;

    /**
     * writes a byte at address.
     */
    fn write_byte(&mut self, address: usize, b: u8) -> Result<(), MemoryError>;

    /**
     * get memory size.
     */
    fn get_size(&self) -> usize;

    /**
     * load file in memory at address.
     */
    fn load(&mut self, path: &str, address: usize) -> Result<(), MemoryError>;
}

/**
 * default implementation of the Memory trait.
 */
struct DefaultMemory {
    size: usize,
    cur: Cursor<Vec<u8>>,
}

impl Memory for DefaultMemory {
    fn read_byte(&mut self, address: usize) -> Result<u8, MemoryError> {
        memory_error::check_address(self, address, 1, MemoryOperation::Read)?;
        self.cur.set_position(address as u64);
        let res = self.cur.read_u8()?;
        Ok(res)
    }

    fn read_word_le(&mut self, address: usize) -> Result<u16, MemoryError> {
        memory_error::check_address(self, address, 2, MemoryOperation::Read)?;

        self.cur.set_position(address as u64);
        let res = self.cur.read_u16::<LittleEndian>()?;
        Ok(res)
    }

    fn write_byte(&mut self, address: usize, b: u8) -> Result<(), MemoryError> {
        memory_error::check_address(self, address, 1, MemoryOperation::Write)?;

        self.cur.set_position(address as u64);
        self.cur.write_u8(b)?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn load(&mut self, path: &str, address: usize) -> Result<(), MemoryError> {
        // check filesize
        let attr = std::fs::metadata(path)?;
        memory_error::check_address(self, address, attr.len() as usize, MemoryOperation::Load)?;

        // read file to a tmp vec
        let mut f = File::open(path)?;
        let mut tmp: Vec<u8> = Vec::new();
        f.read_to_end(&mut tmp)?;

        // read in memory at the given offset
        let m = self.cur.get_mut();
        m.splice(address..attr.len() as usize, tmp);
        Ok(())
    }
}

/**
 * returns an istance of DefaultMemory with the given size.
 */
pub fn new_default(size: usize) -> Box<dyn Memory> {
    // create memory
    let mut m = DefaultMemory {
        size: size,
        cur: Cursor::new(Vec::with_capacity(size)),
    };

    // and fill with zeroes
    let v = m.cur.get_mut();
    for _ in 0..size {
        v.push(0)
    }

    Box::new(m)
}
