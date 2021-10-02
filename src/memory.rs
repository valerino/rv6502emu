use crate::cpu::cpu_error::CpuError;

/*
 * Filename: /src/memory.rs
 * Project: rv6502emu
 * Created Date: 2021-09-20, 08:25:29
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
/**
 * trait for the emulated memory exposed by the cpu.
 *
 */
pub trait Memory {
    /***
     * getter
     */
    fn get_memory(&self) -> &Vec<u8>;

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
     * load file in memory at address. files bigger than 0xffff will be truncated.
     */
    fn load(&mut self, path: &str, address: usize) -> Result<(), CpuError>;

    /**
     * fill memory with zeroes and reset cursor to 0.
     */
    fn clear(&mut self);

    /**
     * gets a reference to the underlying buffer.
     */
    fn as_vec(&self) -> &Vec<u8>;
}
