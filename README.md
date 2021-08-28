# rv6502emu

my toy MOS6502/6510/WDC65C02 cpu emulator + debugger implemented as a rust crate.

> this is my testbed for learning rust, so please sorry if the code is extremely pedantic, probably overengineered (i.e. the [Bus](./src/bus.rs) and [Memory](./src/memory.rs) traits, for instance, made as such to be extensible when used in a console/computer emulator), and most important **may be implemented in a non-idiomatic way due to my current newbieness in rust :)**.<br><br>
said that, **please note that everything (except implementation errors, of course!) is intentional**: i'm trying to experiment with different features of Rust to get a better hold of it and improve my skills.

## features

- __emulates both 6502/6510 and 65C02__
- full featured debugger: 100% (_command-line only currently_)
- undocumented opcodes: 100%
- disassembler : 100%
- assembler : 100%
- emulator : 100%, *including BCD mode for ADC/SBC*, passes **all** [Klaus (functional, decimal, interrupts, 65c02 extended opcodes)](https://github.com/Klaus2m5/6502_65C02_functional_tests) tests.

## usage

here's a [sample program](./src/bin/bin.rs) to use the emulator together with the [Debugger](./src/cpu/debugger.rs) API.

~~~
use rv6502emu::cpu::debugger::Debugger;
use rv6502emu::cpu::{Cpu, CpuType};
use rv6502emu::cpu::CpuCallbackContext;

fn test_callback(_c: &mut Cpu, _cb: CpuCallbackContext) {
    info!("{}", cb);
}

pub fn main() {
    // create a MOS6502(default, also emulates a 6510) cpu with default bus and 64k memory
    let mut c = Cpu::new_default(0x10000, Some(test_callback));

    // enable stdout logger
    c.enable_logging(true);

    let mem = c.bus.get_memory();

    // load test file
    mem.load(
        "./tests/6502_65C02_functional_tests/bin_files/6502_functional_test.bin",
        0,
    )
    .unwrap();

    // you can also change the cpu type at runtime (it's safer to do it only just before calling reset(), either the results are unpredictable!)
    // c.set_cpu_type(CpuType::WDC65C02);

    // resets the cpu (use 0x400 as custom address for the Klaus test) and start execution
    c.reset(Some(0x400)).unwrap();

    // run with a debugger attached, setting an execution breakpoint before starting
    let mut dbg = Debugger::new(true);
    dbg.parse_cmd(&mut c, "bx $404");

    // you can also set breakpoints at specific cycle, i.e. an rw breakpoint
    dbg.parse_cmd(&mut c, "bw $200 cycles=12345");

    // or watching for particular registers ....
    dbg.parse_cmd(&mut c, "bx $200 x=10,y=20");

    // run !
    c.run(Some(&mut dbg), 0).unwrap();
    
    // or, run without debugger attached
    //c.run(None, 0).unwrap();
}
~~~

under debugger CLI, the following features are currently supported via command-line:

~~~
?:> h
debugger supported commands:
        a <$address> .......................... assemble instructions (one per line) at <$address>, <enter> to finish.
        bx|br|bw|brw|bn|bq [$address] [c,...] . add exec/read/write/readwrite/execute/nmi/irq breakpoint, [c]onditions can be <a|x|y|s|p>|<cycles>=n|$n.
        note: for anything except bn and bq, [$address] is mandatory !
        bl .................................... show breakpoints.
        be <n> ................................ enable breakpoint <n>.
        bd <n> ................................ disable breakpoint<n>.
        bdel <n> .............................. delete breakpoint <n>.
        bc .................................... clear all breakpoints.
        c <6502|65C02>......................... switch cpu type (warning: done after reset() may cause unpredictable results !).
        d <# instr> [$address] ................ disassemble <# instructions> at [$address], address defaults to pc.
        e <$value> [$value...] <$address> ..... write one or more <$value> bytes in memory starting at <$address>.
        g ..................................... continue execution until breakpoint or trap.
        h ..................................... this help.
        l <$address> <path> ................... load <path> at <$address>.
        lg .................................... enable/disable cpu log to console (warning, slows down a lot!).
        q ..................................... exit emulator.
        r ..................................... show registers.
        rst [$address] ........................ reset (restart from given [$address], or from address contained at reset vector if empty).
        p ..................................... step next instruction.
        o ..................................... enable/disable show registers before the opcode, default is off (needs logging enabled).
        s <len> <$address> <path> ............. save <len|0=up to memory size> memory bytes starting from <$address> to file at <path>.
        ss .................................... show 16 stack bytes ($1f0-$1ff).
        tn .................................... trigger NMI and set PC=NMI handler.
        tq .................................... trigger IRQ and set PC=IRQ handler.
        v <a|x|y|s|p|pc> <$value>.............. set register value, according to bitness (pc=16bit, others=8bit).
        x <len> <$address> .................... hexdump <len> bytes at <$address>.
NOTE: all addresses/values must be hex where specified, the $ prefix is optional and just for clarity ($0400 = 400). 
        This is valid everywhere but in the handwritten assembler inside the 'a' command.
~~~

~~~bash
git clone <thisrepo> --recurse-submodules

# will run the debugger cli
cargo run
~~~

## status

- need to abstract better the Debugger API to plug a GUI.

cheers :heart:,

v.

