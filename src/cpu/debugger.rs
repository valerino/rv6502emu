/*
 * Filename: /src/debug.rs
 * Project: rv6502emu
 * Created Date: 2021-08-10, 08:46:47
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

use crate::cpu::cpu_error;
use crate::cpu::cpu_error::CpuErrorType;
use crate::cpu::Cpu;
use crate::utils::*;
use breakpoints::Bp;
use hexplay::HexViewBuilder;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::str::SplitWhitespace;

mod asm_disasm;
pub(crate) mod breakpoints;

/**
 * exposes the debugger.
 */
pub struct Debugger {
    /// breakpoints list.
    pub(crate) breakpoints: Vec<Bp>,

    /// debugger enabled/disabled.
    pub enabled: bool,

    /// set by the debugger with the 'g' (continue until break/trap) command.
    pub(crate) going: bool,

    /// to display registers before the opcode.
    pub(crate) show_registers_before_opcode: bool,
}

impl Debugger {
    /**
     * creates a new debugger instance
     */
    pub fn new(enabled: bool) -> Debugger {
        Debugger {
            breakpoints: Vec::new(),
            enabled: enabled,
            going: false,
            show_registers_before_opcode: false,
        }
    }

    /**
     * report invalid command
     */
    fn cmd_invalid(&self) {
        println!("invalid command, try 'h' for help !");
    }

    /**
     * perform cpu reset
     */
    fn cmd_reset(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        let s = it.next().unwrap_or_default();
        if s.len() > 0 {
            // use provided address
            let addr = u16::from_str_radix(&s[is_dollar_hex(&s)..], 16).unwrap_or_default();
            println!("cpu reset, restarting at PC=${:04x}.", addr);
            let _ = match c.reset(Some(addr)) {
                Err(e) => {
                    println!("{}", e);
                    return false;
                }
                Ok(()) => (),
            };
            return true;
        }

        // use the reset vector as default
        println!("cpu reset, restarting at RESET vector.");
        let _ = match c.reset(None) {
            Err(e) => {
                println!("{}", e);
                return false;
            }
            Ok(()) => (),
        };
        return true;
    }

    /**
     * write byte value/s at the given address.
     */
    fn cmd_edit_memory(&self, c: &mut Cpu, it: SplitWhitespace<'_>) -> bool {
        // turn to collection
        let col: Vec<&str> = it.collect();
        let l = col.len();
        if l < 2 {
            // invalid command
            self.cmd_invalid();
            return false;
        }

        // last item is the address
        let addr_s = col[l - 1];
        let mut addr: u16;
        let _ = match u16::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            Ok(a) => addr = a,
        };

        // check access
        let mem = c.bus.get_memory();
        let _ = match cpu_error::check_address_boundaries(
            mem.get_size(),
            addr as usize,
            col.len() - 1,
            CpuErrorType::MemoryWrite,
            None,
        ) {
            Err(e) => {
                println!("{}", e);
                return false;
            }
            Ok(()) => (),
        };

        // write all items starting at address (may overlap)
        println!("writing {} bytes starting at {}.\n", l - 1, addr_s);
        for (i, item) in col.iter().enumerate() {
            if i == (l - 1) {
                break;
            }

            let b: u8;
            let _ = match u8::from_str_radix(&item[is_dollar_hex(&item)..], 16) {
                Err(_) => {
                    // invalid command, value invalid
                    self.cmd_invalid();
                    return false;
                }
                Ok(a) => b = a,
            };
            let _ = match c.bus.get_memory().write_byte(addr as usize, b) {
                Err(e) => {
                    println!("{}", e);
                    return false;
                }
                Ok(_) => {
                    println!("written {} at ${:04x}.", item, addr);
                }
            };

            // next address
            addr = addr.wrapping_add(1);
        }
        return true;
    }

    /**
     * save/hexdump memory
     */
    fn cmd_dump_save_memory(&self, c: &mut Cpu, cmd: &str, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let len_s = it.next().unwrap_or_default();
        let mem = c.bus.get_memory();
        let mut num_bytes = usize::from_str_radix(&len_s, 10).unwrap_or_default();
        if num_bytes == 0 {
            // set to full memory size
            num_bytes = mem.get_size();
        }
        let addr_s = it.next().unwrap_or_default();
        let addr: usize;

        // get the start address
        let _ = match usize::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            Ok(a) => addr = a,
        };

        let mut is_save: bool = false;
        let mut file_path: &str = "";
        if cmd.eq("s") {
            is_save = true;
            // get path
            file_path = it.next().unwrap_or_default();
            if file_path.len() == 0 {
                // invalid command, path invalid
                self.cmd_invalid();
                return false;
            }
        }

        // check access
        let _ = match cpu_error::check_address_boundaries(
            mem.get_size(),
            addr as usize,
            num_bytes as usize,
            CpuErrorType::MemoryRead,
            None,
        ) {
            Err(e) => {
                println!("{}", e);
                return false;
            }
            Ok(()) => (),
        };

        // get the end address
        let addr_end = addr.wrapping_add(num_bytes).wrapping_sub(1);
        let m_slice = &mem.as_vec()[addr as usize..=addr_end as usize];

        if is_save {
            // save to file
            let _ = match File::create(file_path) {
                Err(e) => {
                    // error
                    println!("{}", e);
                    return false;
                }
                Ok(mut f) => {
                    let _ = match f.write_all(m_slice) {
                        Err(e) => {
                            // error
                            println!("{}", e);
                            return false;
                        }
                        Ok(_) => println!("file {} correctly saved!", file_path),
                    };
                }
            };
        } else {
            // dump hex
            let mut sl = vec![0; m_slice.len()];
            sl.copy_from_slice(&m_slice);
            println!("dumping {} bytes at ${:04x}\n", num_bytes, addr);
            let dump = HexViewBuilder::new(&sl)
                .address_offset(addr as usize)
                .row_width(16)
                .finish();
            println!("{}", dump);
        }
        return true;
    }

    /**
     * load file in memory
     */
    fn cmd_load_memory(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let addr_s = it.next().unwrap_or_default();
        let addr: u16;

        let _ = match u16::from_str_radix(&addr_s[is_dollar_hex(&addr_s)..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return false;
            }
            Ok(a) => addr = a,
        };

        // get path
        let file_path = it.next().unwrap_or_default();
        if file_path.len() == 0 {
            // invalid command, path invalid
            self.cmd_invalid();
            return false;
        }
        // clear memory first
        let mem = c.bus.get_memory();
        mem.clear();

        // and load
        match mem.load(file_path, addr as usize) {
            Err(e) => {
                println!("{}", e);
                return false;
            }
            Ok(()) => {}
        };
        return true;
    }

    /**
     * print help banner
     */
    fn cmd_show_help(&self) -> bool {
        println!("debugger supported commands:");
        println!("\ta <$address> .......................... assemble instructions (one per line) at <$address>, <enter> to finish.");
        println!("\tbx|br|bw|brw|bn|bq [$address] [c,...] . add exec/read/write/readwrite/execute/nmi/irq breakpoint, [c]onditions can be <a|x|y|s|p>|<cycles>=n|$n.\n\tnote: for anything except bn and bq, [$address] is mandatory !",
        );
        println!("\tbl .................................... show breakpoints.");
        println!("\tbe <n> ................................ enable breakpoint <n>.");
        println!("\tbd <n> ................................ disable breakpoint<n>.");
        println!("\tbdel <n> .............................. delete breakpoint <n>.");
        println!("\tbc .................................... clear all breakpoints.");
        println!("\td <# instr> [$address] ................ disassemble <# instructions> at [$address], address defaults to pc.",
        );
        println!("\te <$value> [$value...] <$address> ..... write one or more <$value> bytes in memory starting at <$address>.");
        println!(
        "\tg ..................................... continue execution until breakpoint or trap.",
    );
        println!("\th ..................................... this help.");
        println!("\tl <$address> <path> ................... load <path> at <$address>.",);
        println!("\tlg .................................... enable/disable cpu log to console (warning, slows down a lot!).",);
        println!("\tq ..................................... exit emulator.");
        println!("\tr ..................................... show registers.");
        println!("\tp ..................................... step next instruction.");
        println!(
            "\to ..................................... enable/disable show registers before the opcode, default is off (needs logging enabled)."
        );
        println!("\ts <len> <$address> <path> ............. save <len|0=up to memory size> memory bytes starting from <$address> to file at <path>.",
        );
        println!("\tt [$address] .......................... reset (restart from given [$address], or defaults to reset vector).");
        println!("\tv <a|x|y|s|p|pc> <$value>.............. set register value, according to bitness (pc=16bit, others=8bit).");
        println!("\tx <len> <$address> .................... hexdump <len> bytes at <$address>.");
        println!("NOTE: all addresses/values must be hex where specified, the $ prefix is optional and just for clarity ($0400 = 400). 
        This is valid everywhere but in the handwritten assembler inside the 'a' command.");
        return true;
    }

    /**
     * edit cpu registers
     */
    fn cmd_edit_registers(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) -> bool {
        // check input
        let reg = it.next().unwrap_or_default();
        let val = it.next().unwrap_or_default();
        if reg.len() == 0 || val.len() == 0 {
            // invalid command, missing value
            self.cmd_invalid();
            return false;
        }

        // match registers and assign value
        let r = reg.chars().next().unwrap_or_default();
        let res_u16 = u16::from_str_radix(&val[is_dollar_hex(&val)..], 16);
        match r {
            'a' | 'x' | 'y' | 's' | 'p' => match res_u16 {
                Err(_) => {
                    // invalid value
                    self.cmd_invalid();
                    return false;
                }
                Ok(a) => {
                    if reg.eq("pc") {
                        c.regs.pc = a;
                    } else {
                        if a > 0xff {
                            // invalid value
                            self.cmd_invalid();
                            return false;
                        }
                        match r {
                            'a' => c.regs.a = a as u8,
                            'x' => c.regs.x = a as u8,
                            'y' => c.regs.y = a as u8,
                            's' => c.regs.s = a as u8,
                            'p' => c.regs.p = a as u8,
                            _ => (),
                        }
                    }
                }
            },
            _ => {
                // invalid command, register name invalid
                self.cmd_invalid();
                return false;
            }
        }
        println!("register '{}' set to {}.", reg, val);
        return true;
    }

    /**
     * handle debugger input from stdin.
     *
     * returns a tuple with the debugger command ("q" on exit, "*"" for no-op, ...) and a boolean to indicate an error
     */
    pub fn parse_cmd_stdin(&mut self, c: &mut Cpu) -> Result<(String, bool), std::io::Error> {
        if self.enabled {
            if self.going {
                // let it go!
                return Ok((String::from("p"), true));
            }
        }

        // read from stdin
        let mut cmd_string = String::new();
        print!("?:> ");
        io::stdout().flush().unwrap();
        io::stdin().lock().read_line(&mut cmd_string)?;
        Ok(self.parse_cmd(c, &cmd_string))
    }

    /**
     * handle debugger input from string.
     *
     * returns the debugger command ('q' on exit, '*' for no-op)
     */
    pub fn parse_cmd(&mut self, c: &mut Cpu, cmd_string: &str) -> (String, bool) {
        if self.enabled {
            if self.going {
                // let it go!
                return (String::from("p"), true);
            }
        }

        // split command and parameters
        let mut it = cmd_string.split_whitespace();
        let cmd_t = it.next().unwrap_or_default().to_ascii_lowercase();
        let cmd = cmd_t.trim();
        match cmd {
            // assemble
            "a" => {
                return (String::from("*"), self.cmd_assemble(c, it));
            }
            "bc" => {
                return (String::from("*"), self.cmd_clear_breakpoints());
            }
            "be" | "bd" | "bdel" => {
                return (
                    String::from("*"),
                    self.cmd_enable_disable_delete_breakpoint(cmd, it),
                );
            }
            "bx" | "br" | "bw" | "brw" | "bq" | "bn" => {
                return (String::from("*"), self.cmd_add_breakpoint(c, cmd, it));
            }
            "bl" => {
                return (String::from("*"), self.cmd_show_breakpoints());
            }
            // help
            "d" => {
                return (String::from("*"), self.cmd_disassemble(c, it));
            }
            // edit memory
            "e" => {
                return (String::from("*"), self.cmd_edit_memory(c, it));
            }
            // go
            "g" => {
                self.going = true;
                return (String::from("p"), true);
            }
            // help
            "h" => {
                return (String::from("*"), self.cmd_show_help());
            }
            // load memory
            "l" => {
                return (String::from("*"), self.cmd_load_memory(c, it));
            }
            // enable/disable logging
            "lg" => {
                if log_enabled() {
                    c.enable_logging(false);
                    println!("logging is disabled!");
                } else {
                    c.enable_logging(true);
                    println!("logging is enabled!");
                }
                return (String::from("*"), true);
            }
            // quit
            "q" => {
                println!("quit!");
                return (String::from("q"), true);
            }
            // show registers
            "r" => {
                debug_out_registers(c);
                return (String::from("*"), true);
            }
            // step
            "p" => {
                return (String::from("p"), true);
            }
            // show/hide registers before showing the opcode
            "o" => {
                self.show_registers_before_opcode = !self.show_registers_before_opcode;
                println!(
                    "{}showing registers before the opcode.",
                    if self.show_registers_before_opcode {
                        ""
                    } else {
                        "not "
                    }
                );
                return (String::from("*"), true);
            }
            // save memory
            "s" => {
                return (String::from("*"), self.cmd_dump_save_memory(c, cmd, it));
            }
            // reset
            "t" => {
                return (String::from("*"), self.cmd_reset(c, it));
            }
            // edit registers
            "v" => {
                return (String::from("*"), self.cmd_edit_registers(c, it));
            }
            // dump as hex
            "x" => {
                return (String::from("*"), self.cmd_dump_save_memory(c, cmd, it));
            }
            // invalid
            _ => {
                self.cmd_invalid();
                return (String::from("*"), false);
            }
        };
    }
}
