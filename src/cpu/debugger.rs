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

pub(crate) struct Debugger {
    /// breakpoints list.
    pub(crate) breakpoints: Vec<Bp>,

    /// debugger enabled/disabled.
    pub enabled: bool,

    /// set by the debugger with the 'g' (continue until break/trap) command.
    pub(crate) going: bool,
}

impl Debugger {
    /**
     * creates a new debugger instance
     */
    pub(crate) fn new(enabled: bool) -> Debugger {
        Debugger {
            breakpoints: Vec::new(),
            enabled: enabled,
            going: false,
        }
    }

    /**
     * report invalid command
     */
    fn cmd_invalid(&self) {
        debug_out_text(&"invalid command, try 'h' for help !");
    }

    /**
     * perform cpu reset
     */
    fn cmd_reset(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) {
        let s = it.next().unwrap_or_default();
        if s.len() > 0 {
            if s.chars().next().unwrap_or_default() != '$' {
                // invalid
                self.cmd_invalid();
                return;
            }
            // use provided address
            let addr = u16::from_str_radix(&s[1..], 16).unwrap_or_default();
            debug_out_text(&format!("cpu reset, restarting at PC=${:04x}.", addr));
            c.reset(Some(addr)).unwrap_or(());
            return;
        }

        // use the reset vector as default
        debug_out_text(&"cpu reset, restarting at RESET vector.");
        c.reset(None).unwrap_or(());
    }

    /**
     * write byte value/s at the given address.
     */
    fn cmd_edit_memory(&self, c: &mut Cpu, it: SplitWhitespace<'_>) {
        // turn to collection
        let col: Vec<&str> = it.collect();
        let l = col.len();
        if l < 2 {
            // invalid command
            self.cmd_invalid();
            return;
        }

        // all items must start with $
        for item in col.iter() {
            if item.chars().next().unwrap_or_default() != '$' {
                // invalid item
                self.cmd_invalid();
                return;
            }
        }
        // last item is the address
        let addr_s = col[l - 1];
        let mut addr: u16;
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return;
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
                debug_out_text(&e);
                return;
            }
            Ok(()) => (),
        };

        // write all items starting at address (may overlap)
        debug_out_text(&format!(
            "writing {} bytes starting at {}.\n",
            l - 1,
            addr_s
        ));
        for (i, item) in col.iter().enumerate() {
            if i == (l - 1) {
                break;
            }

            let b: u8;
            let _ = match u8::from_str_radix(&item[1..], 16) {
                Err(_) => {
                    // invalid command, value invalid
                    self.cmd_invalid();
                    return;
                }
                Ok(a) => b = a,
            };
            let _ = match c.bus.get_memory().write_byte(addr as usize, b) {
                Err(e) => {
                    debug_out_text(&e);
                    return;
                }
                Ok(_) => {
                    debug_out_text(&format!("written {} at ${:04x}.", item, addr));
                }
            };

            // next address
            addr = addr.wrapping_add(1);
        }
    }

    /**
     * save/hexdump memory
     */
    fn cmd_dump_save_memory(&self, c: &mut Cpu, cmd: &str, mut it: SplitWhitespace<'_>) {
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
        if addr_s.chars().next().unwrap_or_default() != '$' {
            // invalid command, address invalid
            self.cmd_invalid();
            return;
        }
        let _ = match usize::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return;
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
                return;
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
                debug_out_text(&e);
                return;
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
                    debug_out_text(&e);
                    return;
                }
                Ok(mut f) => {
                    let _ = match f.write_all(m_slice) {
                        Err(e) => {
                            // error
                            debug_out_text(&e);
                            return;
                        }
                        Ok(_) => debug_out_text(&"file saved!"),
                    };
                }
            };
        } else {
            // dump hex
            let mut sl = vec![0; m_slice.len()];
            sl.copy_from_slice(&m_slice);
            debug_out_text(&format!("dumping {} bytes at ${:04x}\n", num_bytes, addr));
            let dump = HexViewBuilder::new(&sl)
                .address_offset(addr as usize)
                .row_width(16)
                .finish();

            debug_out_text(&dump);
        }
    }

    /**
     * load file in memory
     */
    fn cmd_load_memory(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) {
        // check input
        let addr_s = it.next().unwrap_or_default();
        let addr: u16;

        // get the start address
        if addr_s.chars().next().unwrap_or_default() != '$' {
            // invalid command, address invalid
            self.cmd_invalid();
            return;
        }
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.cmd_invalid();
                return;
            }
            Ok(a) => addr = a,
        };

        // get path
        let file_path = it.next().unwrap_or_default();
        if file_path.len() == 0 {
            // invalid command, path invalid
            self.cmd_invalid();
            return;
        }
        // clear memory first
        let mem = c.bus.get_memory();
        mem.clear();

        // and load
        match mem.load(file_path, addr as usize) {
            Err(e) => {
                debug_out_text(&e);
                return;
            }
            Ok(()) => debug_out_text(&"file loaded!"),
        };
    }

    /**
     * print help banner
     */
    fn cmd_show_help(&self) {
        println!("debugger supported commands:");
        println!("\ta <$address> .......................... assemble instructions (one per line) at <$address>, <enter> to finish.");
        println!("\tb[x|r|w] .............................. add read/write/execute breakpoint at <$address>.",
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
        println!("\tmi .................................... show memory size.",);
        println!("\tq ..................................... exit emulator.");
        println!("\tr ..................................... show registers.");
        println!("\tp ..................................... step next instruction.");
        println!(
            "\to ......................................step next instruction and show registers."
        );
        println!("\ts <len> <$address> <path> ............. save <len|0=up to memory size> memory bytes starting from <$address> to file at <path>.",
        );
        println!("\tt [$address] .......................... reset (restart from given [$address], or defaults to reset vector).");
        println!("\tv <a|x|y|s|p|pc> <$value>.............. set register value, according to bitness (pc=16bit, others=8bit).");
        println!("\tx <len> <$address> .................... hexdump <len> bytes at <$address>.");
    }

    /**
     * edit cpu registers
     */
    fn cmd_edit_registers(&self, c: &mut Cpu, mut it: SplitWhitespace<'_>) {
        // check input
        let reg = it.next().unwrap_or_default();
        let val = it.next().unwrap_or_default();
        if reg.len() == 0 || val.len() == 0 || val.chars().next().unwrap_or_default() != '$' {
            // invalid command, missing value
            self.cmd_invalid();
            return;
        }

        // match registers and assign value
        let r = reg.chars().next().unwrap_or_default();
        let res_u16 = u16::from_str_radix(&val[1..], 16);
        match r {
            'a' | 'x' | 'y' | 's' | 'p' => match res_u16 {
                Err(_) => {
                    // invalid value
                    self.cmd_invalid();
                    return;
                }
                Ok(a) => {
                    if reg.eq("pc") {
                        c.regs.pc = a;
                    } else {
                        if a > 0xff {
                            // invalid value
                            self.cmd_invalid();
                            return;
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
                return;
            }
        }
        debug_out_text(&format!("register '{}' set to {}.", reg, val));
    }

    /**
     * handle debugger input from stdin.
     *
     * returns the debugger command ('q' on exit, '*' for no-op)
     */
    pub(crate) fn handle_debugger_input_stdin(
        &mut self,
        c: &mut Cpu,
    ) -> Result<char, std::io::Error> {
        if self.enabled {
            if self.going {
                // let it go!
                return Ok('p');
            }
        }
        // read from stdin
        let mut full_string = String::new();
        print!("?:> ");
        io::stdout().flush().unwrap();
        io::stdin().lock().read_line(&mut full_string)?;
        // split command and parameters
        let mut it = full_string.split_whitespace();
        let cmd_t = it.next().unwrap_or_default().to_ascii_lowercase();
        let cmd = cmd_t.trim();
        match cmd {
            // assemble
            "a" => {
                self.cmd_assemble(c, it);
                return Ok('*');
            }
            "bc" => {
                self.cmd_clear_breakpoints();
                return Ok('*');
            }
            "be" | "bd" | "bdel" => {
                self.cmd_enable_disable_delete_breakpoint(cmd, it);
                return Ok('*');
            }
            "bx" | "br" | "bw" | "brw" | "bwr" => {
                self.cmd_add_breakpoint(c, cmd, it);
                return Ok('*');
            }
            "bl" => {
                self.cmd_show_breakpoints();
                return Ok('*');
            }
            // help
            "d" => {
                self.cmd_disassemble(c, it);
                return Ok('*');
            }
            // edit memory
            "e" => {
                self.cmd_edit_memory(c, it);
                return Ok('*');
            }
            // go
            "g" => {
                self.going = true;
                return Ok('p');
            }
            // help
            "h" => {
                self.cmd_show_help();
                return Ok('*');
            }
            // load memory
            "l" => {
                self.cmd_load_memory(c, it);
                return Ok('*');
            }
            // show memory size
            "mi" => {
                let mem_size = c.bus.get_memory().get_size();
                debug_out_text(&format!(
                    "memory size: {} (${:04x}) bytes.",
                    mem_size, mem_size,
                ));
                return Ok('*');
            }
            // quit
            "q" => {
                debug_out_text(&"quit!");
                return Ok('q');
            }
            // show registers
            "r" => {
                debug_out_registers(c);
                return Ok('*');
            }
            // step
            "p" => return Ok('p'),
            // step + show registers
            "o" => return Ok('o'),
            // save memory
            "s" => {
                self.cmd_dump_save_memory(c, cmd, it);
                return Ok('*');
            }
            // reset
            "t" => {
                self.cmd_reset(c, it);
                return Ok('*');
            }
            // edit registers
            "v" => {
                self.cmd_edit_registers(c, it);
                return Ok('*');
            }
            // dump as hex
            "x" => {
                self.cmd_dump_save_memory(c, cmd, it);
                return Ok('*');
            }
            // invalid
            _ => {
                self.cmd_invalid();
                return Ok('*');
            }
        };
    }
}
