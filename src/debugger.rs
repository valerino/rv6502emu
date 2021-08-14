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

use crate::cpu::addressing_modes;
use crate::cpu::addressing_modes::*;
use crate::cpu::cpu_error::CpuError;
use crate::cpu::opcodes;
use crate::cpu::opcodes::OpcodeMarker;
use crate::cpu::Cpu;
use crate::memory::Memory;
use hexplay::HexViewBuilder;
use log::*;
use num;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead, Write};
use std::str::SplitWhitespace;

impl Cpu {
    /**
     * enable debugger
     */
    pub fn enable_debugger(&mut self, enable: bool) {
        self.debug = enable
    }

    /**
     * activate logging on stdout trough env_logger (max level)
     */
    pub fn enable_logging(&self, enable: bool) {
        if enable == true {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::max())
                .try_init();
            log::set_max_level(log::LevelFilter::max());
        } else {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::Off)
                .try_init();
            log::set_max_level(log::LevelFilter::Off);
        }
    }

    /**
     * display opcode string, currently implemented to stdout
     */
    pub(crate) fn debug_out_opcode<A: AddressingMode>(
        &mut self,
        opcode_name: &str,
    ) -> Result<(), CpuError> {
        if log::log_enabled!(Level::max()) || self.debug {
            let opc_string = A::repr(self, opcode_name)?;
            //debug!("\t{}", opc_string);
            println!("\t{}", opc_string);
        }
        Ok(())
    }

    /**
     * display opcode string, currently implemented to stdout
     */
    pub(crate) fn debug_out_text(&self, d: &dyn Display) {
        if log::log_enabled!(Level::max()) || self.debug {
            //debug!("{}", s);
            println!("{}", d);
        }
    }

    /**
     * display registers, currently implemented to stdout
     */
    pub(crate) fn debug_out_registers(&self) {
        if log::log_enabled!(Level::max()) || self.debug {
            //debug!("{}", self.regs);
            println!("\t{}", self.regs);
        }
    }

    /**
     * print help banner
     */
    fn dbg_cmd_show_help(&self) {
        self.debug_out_text(&"debugger supported commands: ");
        self.debug_out_text(&"\ta <$address> .......................... assemble instructions (one per line) at <$address>, x to finish.");
        self.debug_out_text(
                            &"\td <# instr> [$address] ................ disassemble <# instructions> bytes at [$address], defaults to pc.",
        );
        self.debug_out_text(&"\te <$value> [$value...] <$address> ..... write one or more <$value> in memory starting at <$address>.");
        self.debug_out_text(&"\th ..................................... this help.");
        self.debug_out_text(&"\tr ..................................... show registers.");
        self.debug_out_text(
            &"\tp ..................................... step (execute next instruction).",
        );
        self.debug_out_text(&"\tt [$address] .......................... reset (restart from given [$address], or defaults to reset vector).");
        self.debug_out_text(&"\tq ..................................... exit emulator.");
        self.debug_out_text(&"\tv <a|x|y|s|p|pc> <$value>.............. set register value, according to bitness (pc=16bit, others=8bit).");
        self.debug_out_text(
            &"\tx <len> <$address> .................... dump <len> bytes at <$address>.",
        );
    }

    /**
     * perform cpu reset
     */
    fn dbg_cmd_reset(&mut self, mut it: SplitWhitespace<'_>) {
        let s = it.next().unwrap_or_default();
        if s.len() > 0 {
            if s.chars().next().unwrap_or_default() != '$' {
                // invalid
                self.dbg_cmd_invalid();
                return;
            }
            // use provided address
            let addr = u16::from_str_radix(&s[1..], 16).unwrap_or_default();
            self.debug_out_text(&format!("cpu reset, restarting at PC=${:04x}.", addr));
            self.reset(Some(addr)).unwrap_or(());
            return;
        }

        // use the reset vector as default
        self.debug_out_text(&"cpu reset, restarting at RESET vector.");
        self.reset(None).unwrap_or(());
    }

    /**
     * disassemble n instructions at the given address
     */
    fn dbg_disassemble(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let n_s = it.next().unwrap_or_default();
        let n = u16::from_str_radix(&n_s, 10).unwrap_or_default();
        let addr_s = it.next().unwrap_or_default();
        if n == 0 {
            // invalid command, missing number of instructions to decode
            self.dbg_cmd_invalid();
            return;
        }
        // save current pc
        let prev_pc = self.regs.pc;
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            if addr_s.chars().next().unwrap_or_default() != '$' {
                // invalid command, address invalid
                self.dbg_cmd_invalid();
                return;
            }
            match u16::from_str_radix(&addr_s[1..], 16) {
                Err(_) => {
                    // invalid command, address invalid
                    self.dbg_cmd_invalid();
                    return;
                }
                Ok(a) => addr = a,
            }
        } else {
            // defaults to pc
            addr = self.regs.pc;
        }

        // disassemble
        self.regs.pc = addr;
        let mut instr_count = 0;
        self.debug_out_text(&format!(
            "disassembling {} instructions at ${:04x} (may overlap).\n",
            n, addr
        ));
        loop {
            // fetch an instruction
            let b: u8;
            match self.fetch() {
                Err(e) => {
                    self.debug_out_text(&e);
                    return;
                }
                Ok(ok) => b = ok,
            }

            // decode
            let (opcode_f, _, _, _) = opcodes::OPCODE_MATRIX[b as usize];
            let instr_size: i8;
            match opcode_f(self, 0, false, true) {
                Err(e) => {
                    self.debug_out_text(&e);
                    return;
                }
                Ok((a, _)) => instr_size = a,
            }

            instr_count += 1;
            if instr_count == n {
                break;
            }

            // next instruction
            self.regs.pc = self.regs.pc.wrapping_add(instr_size as u16);
        }

        // restore pc in the end
        self.regs.pc = prev_pc;
    }

    /**
     * write byte value/s at the given address.
     */
    fn dbg_write_value(&mut self, mut it: SplitWhitespace<'_>) {
        // turn to collection
        let col: Vec<&str> = it.collect();
        let l = col.len();
        if l < 2 {
            // invalid command
            self.dbg_cmd_invalid();
            return;
        }

        // all items must start with $
        for item in col.iter() {
            if item.chars().next().unwrap_or_default() != '$' {
                // invalid item
                self.dbg_cmd_invalid();
                return;
            }
        }
        // last item is the address
        let addr_s = col[l - 1];
        let mut addr: u16;
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.dbg_cmd_invalid();
                return;
            }
            Ok(a) => addr = a,
        };

        // write all items starting at address (may overlap)
        self.debug_out_text(&format!(
            "writing {} bytes starting at {} (may overlap).\n",
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
                    self.dbg_cmd_invalid();
                    return;
                }
                Ok(a) => b = a,
            };
            let _ = match self.bus.get_memory().write_byte(addr as usize, b) {
                Err(e) => {
                    self.debug_out_text(&e);
                    return;
                }
                Ok(_) => {
                    self.debug_out_text(&format!("written {} at ${:04x}.", item, addr));
                }
            };

            // next address
            addr = addr.wrapping_add(1);
        }
    }

    /**
     * find instruction in the opcode matrix
     */
    fn find_instruction(&self, s: &str, id: AddressingModeId) -> Option<(&OpcodeMarker, u8)> {
        for (i, (_, _, _, op)) in opcodes::OPCODE_MATRIX.iter().enumerate() {
            if op.name.eq(s) && op.id == id {
                return Some((&op, i as u8));
            }
        }
        None
    }

    /**
     * assemble instruction/s
     *
     * syntax for the assembler is taken from https://www.masswerk.at/6502/6502_instruction_set.html
     *
     * A	    Accumulator	        OPC A           operand is AC (implied single byte instruction)
     * abs	    absolute	        OPC $LLHH	    operand is address $HHLL
     * abs,X	absolute, X-indexed	OPC $LLHH,X	    operand is address; effective address is address incremented by X with carry
     * abs,Y	absolute, Y-indexed	OPC $LLHH,Y	    operand is address; effective address is address incremented by Y with carry
     * #	    immediate	        OPC #$BB	    operand is byte BB
     * impl	    implied	            OPC	            operand implied
     * ind	    indirect	        OPC ($LLHH)	    operand is address; effective address is contents of word at address: C.w($HHLL)
     * X,ind	X-indexed, indirect	OPC ($LL,X)	    operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
     * ind,Y	indirect, Y-indexed	OPC ($LL),Y	    operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
     * rel	    relative	        OPC $BB         branch target is PC + signed offset BB
     * zpg	    zeropage	        OPC $LL	        operand is zeropage address (hi-byte is zero, address = $00LL)
     * zpg,X	zeropage, X-indexed	OPC $LL,X	    operand is zeropage address; effective address is address incremented by X without carry
     * zpg,Y	zeropage, Y-indexed	OPC $LL,Y	    operand is zeropage address; effective address is address incremented by Y without carry
     */
    fn dbg_assemble(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let addr_s = it.next().unwrap_or_default();
        let mut addr: u16;
        if addr_s.len() == 0 {
            // invalid command, address invalid
            self.dbg_cmd_invalid();
            return;
        }

        // get the start address
        if addr_s.chars().next().unwrap_or_default() != '$' {
            // invalid command, address invalid
            self.dbg_cmd_invalid();
            return;
        }
        let _ = match u16::from_str_radix(&addr_s[1..], 16) {
            Err(_) => {
                // invalid command, address invalid
                self.dbg_cmd_invalid();
                return;
            }
            Ok(a) => addr = a,
        };

        // read from stdin
        self.debug_out_text(&format!("assembling at ${:04x}, <enter> to stop.", addr));

        // loop
        loop {
            print!("?a> ${:04x}: ", addr);
            io::stdout().flush().unwrap();
            let mut full_string = String::new();
            let _ = match io::stdin().lock().read_line(&mut full_string) {
                Err(_) => break,
                Ok(_) => (),
            };
            // split opcode and operand/s
            full_string = full_string.trim().to_ascii_lowercase();
            if full_string.len() == 0 {
                // done
                break;
            }
            let (mut opcode, tmp) = full_string.split_once(' ').unwrap_or_default();
            opcode = &opcode.trim();

            // also ensure there's no whitestpaces in the operands part
            let mut operand_s = tmp.trim().replace(" ", "").replace("\t", "");

            // find addressing mode and instruction length
            let mode_id: AddressingModeId;
            if operand_s.eq("a") {
                // accumulator
                mode_id = AddressingModeId::Acc;
            } else if operand_s.starts_with("$") && operand_s.len() == 5 && !operand_s.contains(",")
            {
                // absolute
                mode_id = AddressingModeId::Abs;
            } else if operand_s.starts_with("$") && operand_s.ends_with(",x") && operand_s.len() > 6
            {
                // absolute x
                mode_id = AddressingModeId::Abx;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("$") && operand_s.ends_with(",y") && operand_s.len() > 6
            {
                // absolute y
                mode_id = AddressingModeId::Aby;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("#$") {
                // immediate
                mode_id = AddressingModeId::Imm;
                operand_s.remove(0);
            } else if opcode.len() == 0 && operand_s.len() == 0 {
                // implied
                mode_id = AddressingModeId::Imp;
                opcode = &full_string;
            } else if operand_s.starts_with("(") && operand_s.ends_with(")") {
                // indirect
                mode_id = AddressingModeId::Ind;
                operand_s.truncate(operand_s.len() - 1);
                operand_s.remove(0);
            } else if operand_s.ends_with(",x)") {
                // X indirect
                mode_id = AddressingModeId::Xin;
                operand_s.truncate(operand_s.len() - 3);
                operand_s.remove(0);
            } else if operand_s.ends_with("),y") {
                // indirect Y
                mode_id = AddressingModeId::Iny;
                operand_s.truncate(operand_s.len() - 3);
                operand_s.remove(0);
            } else if operand_s.starts_with("$") && operand_s.len() <= 3 {
                if opcode.eq("bpl")
                    || opcode.eq("bmi")
                    || opcode.eq("bvc")
                    || opcode.eq("bvs")
                    || opcode.eq("bcc")
                    || opcode.eq("bcs")
                    || opcode.eq("bne")
                    || opcode.eq("beq")
                {
                    // relative
                    mode_id = AddressingModeId::Rel;
                } else {
                    // zeropage
                    mode_id = AddressingModeId::Zpg;
                }
            } else if operand_s.starts_with("$")
                && operand_s.ends_with(",x")
                && operand_s.len() <= 5
            {
                // zeropage X
                mode_id = AddressingModeId::Zpx;
                operand_s.truncate(operand_s.len() - 2);
            } else if operand_s.starts_with("$")
                && operand_s.ends_with(",y")
                && operand_s.len() <= 5
            {
                // zeropage Y
                mode_id = AddressingModeId::Zpy;
                operand_s.truncate(operand_s.len() - 2);
            } else {
                self.debug_out_text(&"invalid opcode!");
                continue;
            }

            // find a match in the opcode matrix
            let op_byte: u8;
            let _ = match self.find_instruction(&opcode, mode_id) {
                None => {
                    self.debug_out_text(&"invalid opcode!");
                    continue;
                }
                Some((_, idx)) => op_byte = idx,
            };

            /*
            println!(
                "opcode: {} (${:02x}) - operand: {} - modeid={:?}",
                opcode, op_byte, operand_s, mode_id
            );*/

            // write
            #[allow(unused_must_use)]
            match mode_id {
                AddressingModeId::Imp | AddressingModeId::Acc => {
                    self.bus.get_memory().write_byte(addr as usize, op_byte);
                    addr = addr.wrapping_add(1 as u16);
                }
                AddressingModeId::Abs
                | AddressingModeId::Abx
                | AddressingModeId::Aby
                | AddressingModeId::Ind => {
                    let _ = match u16::from_str_radix(&operand_s[1..], 16) {
                        Err(_) => {
                            self.debug_out_text(&"invalid opcode!");
                            continue;
                        }
                        Ok(a) => {
                            self.bus.get_memory().write_byte(addr as usize, op_byte);
                            addr = addr.wrapping_add(1);
                            self.bus.get_memory().write_word_le(addr as usize, a);
                            addr = addr.wrapping_add(2 as u16);
                        }
                    };
                }
                AddressingModeId::Rel
                | AddressingModeId::Imm
                | AddressingModeId::Zpg
                | AddressingModeId::Zpx
                | AddressingModeId::Zpy
                | AddressingModeId::Iny
                | AddressingModeId::Xin => {
                    let _ = match u8::from_str_radix(&operand_s[1..], 16) {
                        Err(_) => {
                            self.debug_out_text(&"invalid opcode!");
                            continue;
                        }
                        Ok(a) => {
                            self.bus.get_memory().write_byte(addr as usize, op_byte);
                            addr = addr.wrapping_add(1);
                            self.bus.get_memory().write_byte(addr as usize, a);
                            addr = addr.wrapping_add(1 as u16);
                        }
                    };
                }
            };
        }
    }

    /**
     * hexdump n bytes at the given address
     */
    fn dbg_dump(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let len_s = it.next().unwrap_or_default();
        let mut num_bytes = u16::from_str_radix(&len_s, 10).unwrap_or_default();
        if num_bytes == 0 {
            // invalid command, missing number of bytes to dump
            self.dbg_cmd_invalid();
            return;
        }
        let addr_s = it.next().unwrap_or_default();
        let addr: u16;

        // get the start address
        if addr_s.len() > 0 {
            if addr_s.chars().next().unwrap_or_default() != '$' {
                // invalid command, address invalid
                self.dbg_cmd_invalid();
                return;
            }
            let _ = match u16::from_str_radix(&addr_s[1..], 16) {
                Err(_) => {
                    // invalid command, address invalid
                    self.dbg_cmd_invalid();
                    return;
                }
                Ok(a) => addr = a,
            };
        } else {
            // defaults to pc
            addr = self.regs.pc;
        }

        // get the end address
        let mut addr_end: u16 = addr.wrapping_add(num_bytes as u16).wrapping_sub(1);
        let mem = self.bus.get_memory();
        if addr_end < addr {
            // address wrapped, use max memory size as the end
            let n_bytes_original = num_bytes;
            num_bytes = (mem.get_size() as u16).wrapping_sub(addr);
            addr_end = addr.wrapping_add(num_bytes).wrapping_sub(1);
            println!("warning: requested {} bytes, dumping {} bytes only due to overlapping with maximum memory size=${:04x}", n_bytes_original, num_bytes, mem.get_size() - 1);
        }
        let m_slice = &mem.as_vec()[addr as usize..=addr_end as usize];

        // dump!
        // MAYBE_FIX: if not using a copy, borrow checker complains of mutable reference to *self used twice (due to self.bus.get_memory())
        let mut sl = vec![0; m_slice.len()];
        sl.copy_from_slice(&m_slice);
        self.debug_out_text(&format!("dumping {} bytes at ${:04x}\n", num_bytes, addr));
        let dump = HexViewBuilder::new(&sl)
            .address_offset(addr as usize)
            .row_width(16)
            .finish();

        self.debug_out_text(&dump);
    }

    /**
     * edit cpu registers
     */
    fn dbg_edit_regs(&mut self, mut it: SplitWhitespace<'_>) {
        // check input
        let reg = it.next().unwrap_or_default();
        let val = it.next().unwrap_or_default();
        if reg.len() == 0 || val.len() == 0 || val.chars().next().unwrap_or_default() != '$' {
            // invalid command, missing value
            self.dbg_cmd_invalid();
            return;
        }

        // match registers and assign value
        let c = reg.chars().next().unwrap_or_default();
        let res_u16 = u16::from_str_radix(&val[1..], 16);
        match c {
            'a' | 'x' | 'y' | 's' | 'p' => match res_u16 {
                Err(_) => {
                    // invalid value
                    self.dbg_cmd_invalid();
                    return;
                }
                Ok(a) => {
                    if reg.eq("pc") {
                        self.regs.pc = a;
                    } else {
                        if a > 0xff {
                            // invalid value
                            self.dbg_cmd_invalid();
                            return;
                        }
                        match c {
                            'a' => self.regs.a = a as u8,
                            'x' => self.regs.x = a as u8,
                            'y' => self.regs.y = a as u8,
                            's' => self.regs.s = a as u8,
                            'p' => self.regs.p = a as u8,
                            _ => (),
                        }
                    }
                }
            },
            _ => {
                // invalid command, register name invalid
                self.dbg_cmd_invalid();
                return;
            }
        }
        self.debug_out_text(&format!("register '{}' set to {}.", reg, val));
    }

    /**
     * report invalid command
     */
    fn dbg_cmd_invalid(&self) {
        self.debug_out_text(&"invalid command, try 'h' for help !");
    }

    /**
     * handle debugger input from stdin, if debugger is active.
     *
     * returns the debugger command ('q' on exit, '*' for no-op)
     */
    pub(crate) fn handle_debugger_input_stdin(&mut self) -> Result<char, std::io::Error> {
        if self.debug {
            // read from stdin
            let mut full_string = String::new();
            print!("?:> ");
            io::stdout().flush().unwrap();
            io::stdin().lock().read_line(&mut full_string)?;

            // split command and parameters
            let mut it = full_string.split_whitespace();
            let cmd = it.next().unwrap_or_default().to_ascii_lowercase();

            // handle command
            match cmd.trim() {
                // assemble
                "a" => {
                    self.dbg_assemble(it);
                    return Ok('*');
                }
                // help
                "d" => {
                    self.dbg_disassemble(it);
                    return Ok('*');
                }
                // edit memory
                "e" => {
                    self.dbg_write_value(it);
                    return Ok('*');
                }
                // help
                "h" => {
                    self.dbg_cmd_show_help();
                    return Ok('*');
                }
                // quit
                "q" => {
                    self.debug_out_text(&"quit!");
                    return Ok('q');
                }
                // show registers
                "r" => {
                    self.debug_out_registers();
                    return Ok('*');
                }
                // step
                "p" => return Ok('p'),
                // reset
                "t" => {
                    self.dbg_cmd_reset(it);
                    return Ok('*');
                }
                // edit registers
                "v" => {
                    self.dbg_edit_regs(it);
                    return Ok('*');
                }
                "x" => {
                    self.dbg_dump(it);
                    return Ok('*');
                }
                // invalid
                _ => {
                    self.dbg_cmd_invalid();
                    return Ok('*');
                }
            }
        }

        // default returns 'no-op'
        Ok('*')
    }
}
