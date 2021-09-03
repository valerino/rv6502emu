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
static TRIG_IRQ_BIT: u8 = 0;
static TRIG_NMI_BIT: u8 = 1;
static SRC_BRK_BIT: u8 = 0;
static SRC_IRQ_BIT: u8 = 1;
static SRC_NMI_BIT: u8 = 2;

fn check_int_src(c: &mut Cpu, val: u8) -> (bool, bool, bool, bool) {
    let mut must_trigger_irq = false;
    let mut must_trigger_nmi = false;
    let mut pending_irq = false;
    let mut pending_nmi = false;

    // get interrupt source
    let src = c.bus.get_memory().read_byte(0x203).unwrap();
    let v = val & 0x7f;
    let mut is_brk = false;
    println!("src={:02x},v={:02x},pc={:04x} !", src, v, c.regs.pc);

    if (src & (1 << SRC_BRK_BIT)) != 0 {
        println!("BRK src at pc={:04x} !", c.regs.pc);
    }

    if v & (1 << TRIG_IRQ_BIT) != 0 {
        if src & (1 << SRC_IRQ_BIT) != 0 {
            must_trigger_irq = true;
        } else {
            if src != 0 {
                pending_irq = true;
            }
        }
    }
    if v & (1 << TRIG_NMI_BIT) != 0 {
        if src & (1 << SRC_NMI_BIT) != 0 {
            must_trigger_nmi = true;
        } else {
            if src != 0 {
                pending_nmi = true;
            }
        }
    }
    (must_trigger_irq, must_trigger_nmi, pending_irq, pending_nmi)
}

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
        } else if TEST == 2 {
            if cb.operation == CpuOperation::Write && !c.processing_ints {
                if cb.address == 0xbffc {
                    let (trigger_irq, trigger_nmi, pending_irq, pending_nmi) =
                        check_int_src(c, cb.value);
                    if trigger_irq {
                        println!("adding irq at pc={:04x} !", c.regs.pc);
                        c.add_irq(false);
                    }
                    if pending_irq {
                        println!("adding pending irq at pc={:04x} !", c.regs.pc);
                        c.add_irq(true);
                    }
                    if trigger_nmi {
                        println!("adding nmi at pc={:04x} !", c.regs.pc);
                        c.add_nmi(false);
                    }
                    if pending_nmi {
                        println!("adding pending nmi at pc={:04x} !", c.regs.pc);
                        c.add_nmi(true);
                    }
                }
            } else if cb.operation == CpuOperation::Exec {
                if c.regs.pc == 0x6f5 {
                    println!(
                        "yay! PC=${:04x}, Klaus interrupt test SUCCEEDED !",
                        c.regs.pc
                    );
                    // done!
                    c.done = true;
                }
            }
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
