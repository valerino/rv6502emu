/*
 * Filename: /tests/test.rs
 * Project: rv6502emu
 * Created Date: 2021-08-09, 01:10:30
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

use log::*;
use rv6502emu::cpu::Cpu;
use rv6502emu::cpu::CpuCallbackContext;
use rv6502emu::memory::Memory;
fn test_inner(mem: &mut Box<dyn Memory>) {
    let b = mem.read_byte(123).unwrap();
    info!("b after write 2={:x}", b);
    assert_eq!(b, 0xfc);
}

fn test_read_writes(mem: &mut Box<dyn Memory>) {
    // some read and writes
    let mut bb = mem.read_byte(123).unwrap();
    info!("b after read ={}", bb);
    assert_eq!(bb, 0xff);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(123).unwrap();
    info!("b after write 1={:x}", bb);
    assert_eq!(bb, 0xaa);

    // some read and writes in a function
    mem.write_byte(123, 0xfc);
    test_inner(mem);

    let b = mem.read_byte(123).unwrap();
    assert_eq!(b, 0xfc)
}

fn test_callback(cb: CpuCallbackContext) {
    info!("hello from callback {:?}", cb);
}

/**
 * tests the cpu using klaus test (https://github.com/Klaus2m5/6502_65C02_functional_tests)
 */
#[test]
fn test_cpu() {
    // init logger
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .try_init();

    // create a cpu with default bus and 64k memory
    let mut c = Cpu::new_default(0x10000, Some(test_callback), true);
    let mem = c.bus.get_memory();

    // load test file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();

    // resets the cpu (use 0x400 as custom address for the Klaus test) and start execution
    c.reset(Some(0x400));
    c.run(0);

    /*
    // create a cpu with default bus and 64k memory
    let mut c = rv6502emu::cpu::Cpu::new_default(0x10000);
    let mem = c.bus.get_memory();
    // test_read_writes(mem);

    // load test file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();

    // resets the cpu and start execution
    c.reset();

    info!("cpu thread handle={:?}", std::thread::current());
    let chan_for_ui = c.chan.clone();
    let mut dbg_ui = rv6502emu::gui::new(chan_for_ui);
    /*
    cb_thread::scope(|scope| {
        scope.spawn(move |_| {
            //dbg_ui.start_comm_thread();
            info!("running gui");
            dbg_ui.run();
        });
    })
    .unwrap();
    */
    std::thread::spawn(move || {
        info!("running gui");
        dbg_ui.run();
    });
    loop {
        info!("waiting ...");
        let now = std::time::Instant::now();
        let sec = std::time::Duration::from_secs(1);
        while now.elapsed() < sec {
            std::thread::yield_now();
        }
    }

    //dbg_ui.start_comm_thread();
    //info!("running gui");
    //dbg_ui.run();
    */
    /*
    //let mut dbg_ui = rv6502emu::gui::new(&c.to_ui_channels, &c.from_ui_channels);
    //let t_handle = dbg_ui.run();
    info!("receiving.....");
    c.from_ui_channels.1.recv();
    info!("received!");
    t_handle.join();
    */
    /*
    // some read and writes
    let mut bb = mem.read_byte(123).unwrap();
    info!("b after read ={}", bb);
    mem.write_byte(123, 0xaa);

    // read again
    bb = mem.read_byte(123).unwrap();
    info!("b after write={:x}", bb);

    // some read and writes in a function
    tt(mem);
    mem.write_byte(123, 0xfc);
    tt(mem)
    */

    // run test
    /*
    loop {
        c.step(0);
    }*/
}
