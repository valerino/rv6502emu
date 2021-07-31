/*
 * Filename: /src/bus.rs
 * Project: rv6502emu
 * Created Date: Thursday, January 1st 1970, 1:00:00 am
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

/**
 * a Bus is connected to the Cpu, and must expose at least a Memory interface.
 */
pub trait Bus {
    /**
     * gets the emulated memory.
     */
    fn get_memory(&mut self) -> &mut Box<dyn Memory>;
}

/**
 * implements the default Bus exposing Memory only.
 */
struct DefaultBus {
    m: Box<dyn Memory>,
}

impl Bus for DefaultBus {
    fn get_memory(&mut self) -> &mut Box<dyn Memory> {
        &mut self.m
    }
}

/**
 * creates a new default bus with the given Memory attached.
 */
pub fn new_default(mem: Box<dyn Memory>) -> Box<dyn Bus> {
    let b = DefaultBus { m: mem };
    Box::new(b)
}
