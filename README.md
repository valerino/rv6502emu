# rv6502emu

my toy MOS6502 cpu emulator implemented as a rust crate.

> this is my testbed for learning rust, so please sorry if the code is extremely pedantic, probably overengineered (i.e. the [Bus](./src/bus.rs) and [Memory](./src/memory.rs) traits, for instance, made as such to be extensible when used in a console/computer emulator), and most important **may be implemented in a non-idiomatic way due to my current newbieness in rust :)**.<br><br>
said that, **please note that everything (except implementation errors, of course!) is intentional**: i'm trying to experiment with different features of Rust to get a better hold of it and improve my skills.<br><br>
hopefully this will work too once finished, i plan to use it for a rust-based Atari2600 emulator :)

## usage

at the moment, the project is not finished yet.

- the instruction decoder seems working (implementing a debugger should be easy now).
- the different addressing modes seems working and there's a bit of output.
- i'm currently implementing the opcodes now....

to get a hold, i'm implementing an [integration test](./tests/test.rs) using [klaus test](https://github.com/Klaus2m5/6502_65C02_functional_tests) to stress-test the implementation.

~~~bash
git clone <thisrepo> --recurse-submodules
cargo test
~~~

cheers :heart:,

v.

