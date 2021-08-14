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

use crate::cpu::cpu_error;
use crate::cpu::cpu_error::{CpuError, CpuErrorType};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
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
    fn read_byte(&mut self, address: usize) -> Result<u8, CpuError>;

    /**
     * reads a word (little-endian) at address.
     */
    fn read_word_le(&mut self, address: usize) -> Result<u16, CpuError>;

    /**
     * writes a word (little-endian) at address.
     */
    fn write_word_le(&mut self, address: usize, w: u16) -> Result<(), CpuError>;

    /**
     * writes a byte at address.
     */
    fn write_byte(&mut self, address: usize, b: u8) -> Result<(), CpuError>;

    /**
     * get memory size.
     */
    fn get_size(&self) -> usize;

    /**
     * load file in memory at address.
     */
    fn load(&mut self, path: &str, address: usize) -> Result<(), CpuError>;

    /**
     * gets a reference to the underlying buffer.
     */
    fn as_vec(&self) -> &Vec<u8>;
}

/**
 * default implementation of the Memory trait.
 */
struct DefaultMemory {
    size: usize,
    cur: Cursor<Vec<u8>>,
}

impl Memory for DefaultMemory {
    fn as_vec(&self) -> &Vec<u8> {
        let v = self.cur.get_ref();
        v
    }
    fn read_byte(&mut self, address: usize) -> Result<u8, CpuError> {
        cpu_error::check_address(self, address, 1, CpuErrorType::MemoryRead)?;
        self.cur.set_position(address as u64);
        let res = self.cur.read_u8()?;
        Ok(res)
    }

    fn read_word_le(&mut self, address: usize) -> Result<u16, CpuError> {
        cpu_error::check_address(self, address, 2, CpuErrorType::MemoryRead)?;

        self.cur.set_position(address as u64);
        let res = self.cur.read_u16::<LittleEndian>()?;
        Ok(res)
    }

    fn write_word_le(&mut self, address: usize, w: u16) -> Result<(), CpuError> {
        cpu_error::check_address(self, address, 2, CpuErrorType::MemoryWrite)?;
        self.cur.set_position(address as u64);
        let res = self.cur.write_u16::<LittleEndian>(w)?;
        Ok(res)
    }

    fn write_byte(&mut self, address: usize, b: u8) -> Result<(), CpuError> {
        cpu_error::check_address(self, address, 1, CpuErrorType::MemoryWrite)?;

        self.cur.set_position(address as u64);
        self.cur.write_u8(b)?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn load(&mut self, path: &str, address: usize) -> Result<(), CpuError> {
        // check filesize
        let attr = std::fs::metadata(path)?;
        cpu_error::check_address(self, address, attr.len() as usize, CpuErrorType::MemoryLoad)?;

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
