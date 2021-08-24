use rv6502emu::cpu::debugger::Debugger;
use rv6502emu::cpu::CpuCallbackContext;
use rv6502emu::cpu::{Cpu, CpuOperation};

static mut TEST: i8 = 0;

fn test_callback(c: &mut Cpu, cb: CpuCallbackContext) {
    // check final PC for klaus functional test
    unsafe {
        if TEST == 0 && c.regs.pc == 0x3469 && cb.operation == CpuOperation::Exec {
            println!("yay! PC=$3469 hit, Klaus functional test SUCCEEDED !");
            c.done = true;
        } else if TEST == 1 && c.regs.pc == 0x24b && cb.operation == CpuOperation::Exec {
            // read ERROR
            if c.bus.get_memory().read_byte(0xb).unwrap() == 0 {
                println!("yay! PC=$24b hit, Bruce Clark decimal test SUCCEDED!");
            } else {
                println!("PC=$24b hit, Bruce Clark decimal test FAILED!");
            }
            c.done = true;
        }
    }
}

pub fn main() {
    // create a cpu with default bus, including max addressable memory (64k)
    let mut c = Cpu::new_default(Some(test_callback));

    // enable stdout logger
    c.enable_logging(true);

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

    // run with a debugger attached
    let mut dbg = Debugger::new(true);
    c.run(Some(&mut dbg), 0).unwrap();

    // load decimal test
    unsafe {
        TEST = 1;
    }
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
    c.run(Some(&mut dbg), 0).unwrap();

    // or, run without debugger attached
    //c.run(None, 0).unwrap();
}
