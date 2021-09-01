/*
 * Filename: /src/bin/bin.rs
 * Project: rv6502emu
 * Created Date: 2021-08-25, 12:18:22
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

use rv6502emu::cpu::debugger::Debugger;
use rv6502emu::cpu::CpuCallbackContext;
use rv6502emu::cpu::{Cpu, CpuOperation, CpuType};

static mut TEST: i8 = 0;

fn test_callback(c: &mut Cpu, cb: CpuCallbackContext) {
    // check final PC for klaus functional test
    unsafe {
        if TEST == 0 && c.regs.pc == 0x3469 && cb.operation == CpuOperation::Exec {
            println!(
                "yay! PC=${:04x}, Klaus functional test SUCCEEDED !",
                c.regs.pc
            );
            // done!
            c.done = true;
        } else if TEST == 1 && c.regs.pc == 0x24b && cb.operation == CpuOperation::Exec {
            if c.bus.get_memory().read_byte(0xb).unwrap() == 0 {
                println!(
                    "yay! PC=${:04x} hit, Bruce Clark decimal test SUCCEDED!",
                    c.regs.pc
                );
            } else {
                println!(
                    ":( PC=${:04x} hit, Bruce Clark decimal test FAILED!",
                    c.regs.pc
                );
            }
            // done!
            c.done = true;
        } else if TEST == 2 && cb.operation == CpuOperation::Exec {
            // we trigger irq/nmi at certain PC to simulate interrupts
            // refer to tests/6502_65C02_functional_tests/bin_files/6502_interrupt_test.lst for addresses
            match c.regs.pc {
                0x42b | 0x45d | 0x49e | 0x4db => {
                    c.must_trigger_irq = true;
                }
                0x5c7 | 0x5f9 | 0x63a | 0x677 | 0x6a7 | 0x6e4 => {
                    c.must_trigger_nmi = true;
                    if c.regs.pc == 0x6a7 || c.regs.pc == 0x6e4 {
                        // also trigger irq
                        c.must_trigger_irq = true;
                    }
                }
                0x6f5 => {
                    println!(
                        "yay! PC=${:04x}, Klaus interrupt test SUCCEEDED !",
                        c.regs.pc
                    );
                    // done!
                    c.done = true;
                }
                _ => (),
            };
        } else if TEST == 3 && c.regs.pc == 0x24f1 && cb.operation == CpuOperation::Exec {
            println!(
                "yay! PC=${:04x}, Klaus 65C02 extended opcodes test SUCCEEDED !",
                c.regs.pc
            );
            // done!
            c.done = true;
        }
    }
}

fn decimal_test(c: &mut Cpu, d: Option<&mut Debugger>) {
    unsafe {
        TEST = 1;
    }

    // load decimal test
    c.bus
        .get_memory()
        .load(
            "./tests/6502_65C02_functional_tests/bin_files/6502_decimal_test.bin",
            0x200,
        )
        .unwrap();

    // resets to $200
    c.reset(Some(0x200)).unwrap();

    // and run again
    c.run(d, 0).unwrap();
}

fn interrupt_test(c: &mut Cpu, d: Option<&mut Debugger>) {
    unsafe {
        TEST = 2;
    }

    // load interrupts test
    c.bus
        .get_memory()
        .load(
            "./tests/6502_65C02_functional_tests/bin_files/6502_interrupt_test.bin",
            0xa,
        )
        .unwrap();

    // resets to $400
    c.reset(Some(0x400)).unwrap();
    let mut empty_dbg = Debugger::new(false);
    let dbg = d.unwrap_or(&mut empty_dbg);

    // and run
    c.run(Some(dbg), 0).unwrap();
}

/**
 * runs the klaus functional test
 */
fn klaus_functional_test(c: &mut Cpu, d: Option<&mut Debugger>) {
    unsafe {
        TEST = 0;
    }

    // load klaus functional test to memory
    c.bus
        .get_memory()
        .load(
            "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
            0,
        )
        .unwrap();

    // resets the cpu (use 0x400 as custom address for the Klaus test) and start execution
    c.reset(Some(0x400)).unwrap();

    // run
    c.run(d, 0).unwrap();
}

/**
 * runs the klaus functional test
 */
fn klaus_65c02_test(c: &mut Cpu, d: Option<&mut Debugger>) {
    unsafe {
        TEST = 3;
    }

    // set cpu to 65c02
    c.set_cpu_type(CpuType::WDC65C02);

    // load test to memory
    c.bus
        .get_memory()
        .load(
            "./tests/6502_65C02_functional_tests/bin_files/65C02_extended_opcodes_test.bin",
            0,
        )
        .unwrap();

    // resets the cpu
    c.reset(Some(0x400)).unwrap();

    // run
    c.run(d, 0).unwrap();
}

pub fn main() {
    // create a cpu with default bus, including max addressable memory (64k)
    let mut c = Cpu::new_default(Some(test_callback));
    c.enable_logging(false);
    // create a debugger
    let mut dbg = Debugger::new(true);

    // run tests
    klaus_functional_test(&mut c, Some(&mut dbg));
    decimal_test(&mut c, Some(&mut dbg));
    interrupt_test(&mut c, Some(&mut dbg));
    klaus_65c02_test(&mut c, Some(&mut dbg));
}
