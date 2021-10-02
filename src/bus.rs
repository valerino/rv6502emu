/*
 * Filename: /src/newbus.rs
 * Project: rv6502emu
 * Created Date: 2021-09-20, 08:21:22
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

use crate::{cpu::cpu_error::CpuError, memory::Memory};

pub struct Bus {
    mem_size: usize,
    m: Vec<u8>,
}

impl Bus {
    /**
     * creates a new Bus with 64k memory.
     */
    pub fn new() -> Self {
        let b = Bus {
            mem_size: 0x10000, // 64k max
            m: Vec::new(),
        };
        for i in 0..b.mem_size {
            b.m.push(0x0)
        }
        b
    }
}

impl Memory for Bus {
    fn read_byte(&mut self, address: usize) -> Result<u8, CpuError> {
        todo!()
    }

    fn read_word_le(&mut self, address: usize) -> Result<u16, CpuError> {
        todo!()
    }

    fn write_word_le(&mut self, address: usize, w: u16) -> Result<(), CpuError> {
        todo!()
    }

    fn write_byte(&mut self, address: usize, b: u8) -> Result<(), CpuError> {
        todo!()
    }

    fn get_size(&self) -> usize {
        todo!()
    }

    fn load(&mut self, path: &str, address: usize) -> Result<(), CpuError> {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn as_vec(&self) -> &Vec<u8> {
        todo!()
    }
}
