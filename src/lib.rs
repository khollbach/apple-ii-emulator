#![allow(unused_imports)] // todo

use std::ops::ControlFlow;

use commands::Command;
use cpu::{instr::Instr, Cpu};
use display::{color::Color, gr, hgr, text};
use itertools::Itertools;
use memory::Memory;

pub mod commands;
mod cpu;
mod display;
pub mod gui;
pub mod hex;
mod memory;

pub struct Emulator {
    cpu: Cpu,
    mem: Memory,
    halted: bool,
    num_instructions_executed: u64,
    breakpoints: Vec<u16>,
}

impl Emulator {
    pub fn new(program: &[u8], load_addr: u16, start_addr: u16) -> Self {
        let mut mem = Memory::new(&program, load_addr);
        let pc = mem.set_softev(start_addr);

        Self {
            cpu: Cpu::new(pc),
            mem,
            halted: false,
            num_instructions_executed: 0,
            breakpoints: vec![],
        }
    }

    /// Called at ~300 Hz.
    pub fn sim_1000_instrs(&mut self) {
        if self.halted {
            return;
        }

        for _ in 0..1_000 {
            if self.cpu.next_instr(&mut self.mem).0 == Instr::Brk {
                eprintln!("\n\n>>> would break");
                self.halted = true;
                break;
            }
            if self.cpu.would_halt(&mut self.mem) {
                eprintln!("\n\n>>> would halt");
                self.halted = true;
                break;
            }
            if self.breakpoints.contains(&self.cpu.pc) {
                eprintln!("\n\n>>> breakpoint");
                self.halted = true;
                break;
            }

            self.cpu.step(&mut self.mem);
            self.num_instructions_executed += 1;

            // Detect long-running loops that aren't a simple "halt
            // instruction".
            let i = self.num_instructions_executed;
            let thresh = 100_000_000;
            if i != 0 && i % thresh == 0 {
                let m = 1_000_000;
                eprintln!("\n\n>>> after {}M instructions,", i / m);
                eprintln!(">>> {:?}\n", self.cpu);
                eprint!("... ");
            }
        }

        if self.halted {
            let cpu_info = self.cpu.dbg(&mut self.mem).to_string();
            for line in cpu_info.lines() {
                eprintln!(">>> {line}");
            }
            eprintln!();
            eprint!("... ");
        }
    }

    /// Called at 60 Hz.
    //
    // todo:
    // * change output type to [[Color; 280]; 192] ?
    // * make output to &mut param ?
    pub fn draw_screen(&self) -> Vec<Vec<Color>> {
        // TODO at some point:
        // * maybe render all 3 screens, for easier debugging ?

        // todo:
        // * impl soft switches for toggling b/w (fullscreen) display modes
        // * (mixed mode is much lower prio)

        // ignore unused code
        let _ = gr::dots(self.mem.gr_page1());
        let _ = hgr::dots_color(self.mem.hgr_page1());
        let _ = hgr::dots_bw(self.mem.hgr_page1());

        text::dots(self.mem.gr_page1())
    }

    pub fn key_down(&mut self, ascii_code: u8) {
        self.mem.key_down(ascii_code);
    }

    pub fn all_keys_up(&mut self) {
        todo!("clear any-key-down flag");
        // TODO: also make sure we're actually setting said flag in mem.key_down !
    }

    /// Execute a "debugger" command.
    pub fn control(&mut self, cmd: Command) {
        match cmd {
            Command::Halt => {
                if self.halted {
                    println!("already halted");
                } else {
                    self.halted = true;
                    println!("{}\n", self.cpu.dbg(&mut self.mem));
                }
            }
            Command::Continue => {
                if !self.halted {
                    println!("already running");
                } else {
                    self.halted = false;
                }
            }
            Command::Step => {
                if !self.halted {
                    println!("halting");
                    self.halted = true;
                }
                self.cpu.step(&mut self.mem);
                self.num_instructions_executed += 1;

                println!("{}\n", self.cpu.dbg(&mut self.mem));
            }
            Command::ToggleBreakpoint { addr } => {
                if let Some((idx, _)) = self.breakpoints.iter().find_position(|&&a| a == addr) {
                    self.breakpoints.swap_remove(idx);
                    println!("cleared breakpoint ${:04x}", addr);
                } else {
                    self.breakpoints.push(addr);
                    println!("set breakpoint ${:04x}", addr);
                }
            }
            Command::ShowByte { addr } => {
                println!("ram[${:04x}]: ${:02x}\n", addr, self.mem.get(addr));
            }
            Command::ShowRange {
                start,
                end_inclusive,
            } => {
                println!("ram[${:04x}..=${:04x}]:", start, end_inclusive);
                for addr in start..=end_inclusive {
                    print!("{:02x} ", self.mem.get(addr));
                }
                println!("\n");
            }
        }
    }
}
