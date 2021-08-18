# rv6502emu

my toy MOS6502 cpu emulator + debugger implemented as a rust crate.

> this is my testbed for learning rust, so please sorry if the code is extremely pedantic, probably overengineered (i.e. the [Bus](./src/bus.rs) and [Memory](./src/memory.rs) traits, for instance, made as such to be extensible when used in a console/computer emulator), and most important **may be implemented in a non-idiomatic way due to my current newbieness in rust :)**.<br><br>
said that, **please note that everything (except implementation errors, of course!) is intentional**: i'm trying to experiment with different features of Rust to get a better hold of it and improve my skills.<br><br>
hopefully this works too, i plan to use it for a rust-based Atari2600 emulator :)

## features

- full featured debugger: 100% (_command-line only currently_)
- undocumented opcodes: 100%
- disassembler : 100%
- assembler : 100%
- emulator : 100%

## usage

~~~
    /**
     * creates a new cpu instance, with the given Bus attached.
     */
    pub fn new(
        b: Box<dyn Bus>,
        cb: Option<fn(c: &mut Cpu, cb: CpuCallbackContext)>,
        debug: bool,
    ) -> Cpu

    /**
     * creates a new cpu instance, with the given Bus attached, exposing a Memory of the given size.
     */
    pub fn new_default(
        mem_size: usize,
        cb: Option<fn(c: &mut Cpu, cb: CpuCallbackContext)>,
        debug: bool,
    ) -> Cpu

    /**
     * resets the cpu setting all registers to the initial values.
     * http://forum.6502.org/viewtopic.php?p=2959
     */
    pub fn reset(&mut self, start_address: Option<u16>) -> Result<(), CpuError>

    /**
     * run the cpu for the given cycles, pass 0 to run indefinitely.
     *
     * > note that reset() must be called first to set the start address !
     */
     pub fn run(&mut self, cycles: usize) -> Result<(), CpuError> {
    
~~~

under debugger (debug=true), the following features are currently supported via command-line:

~~~
?:> h
debugger supported commands:
        a <$address> .......................... assemble instructions (one per line) at <$address>, <enter> to finish.
        b[x|r|w] .............................. add read/write/execute breakpoint at <$address>.
        bl .................................... show breakpoints.
        be <n> ................................ enable breakpoint <n>.
        bd <n> ................................ disable breakpoint<n>.
        bdel <n> .............................. delete breakpoint <n>.
        bc .................................... clear all breakpoints.
        d <# instr> [$address] ................ disassemble <# instructions> at [$address], address defaults to pc.
        e <$value> [$value...] <$address> ..... write one or more <$value> bytes in memory starting at <$address>.
        g ..................................... continue execution until breakpoint or trap.
        h ..................................... this help.
        l <$address> <path> ................... load <path> at <$address>.
        mi .................................... show memory size.
        q ..................................... exit emulator.
        r ..................................... show registers.
        p ..................................... step next instruction.
        s <len> <$address> <path> ............. save <len|0=up to memory size> memory bytes starting from <$address> to file at <path>.
        t [$address] .......................... reset (restart from given [$address], or defaults to reset vector).
        v <a|x|y|s|p|pc> <$value>.............. set register value, according to bitness (pc=16bit, others=8bit).
        x <len> <$address> .................... hexdump <len> bytes at <$address>.
~~~

~~~bash
git clone <thisrepo> --recurse-submodules

# will run the debugger cli
cargo run
~~~

cheers :heart:,

v.

