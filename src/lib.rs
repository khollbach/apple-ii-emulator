#![allow(unused_imports)] // todo

use std::ops::ControlFlow;

use cpu::{instr::Instr, Cpu};
use debugger_commands::Command;
use display::{color::Color, gr, hgr, text};
use itertools::Itertools;
use memory::AddressSpace;

mod cpu;
pub mod debugger_commands;
mod display;
pub mod gui;
pub mod hex;
mod memory;

pub struct Emulator {
    cpu: Cpu,
    mem: AddressSpace,
    halted: bool,
    num_instructions_executed: u64,
    breakpoints: Vec<u16>,
    /// If a `finish` command is ongoing, this stores the current subroutine
    /// depth, e.g.:
    /// * 0 if we haven't called any inner subroutines
    /// * 3 if we're 3 subroutines deep
    /// And when it would go negative, we know we've returned from the top-level
    /// subroutine.
    ///
    /// NB: the Apple IIe 80col ROM uses hacks and tricks (like RTS without a
    /// JSR), so this will sometimes halt earlier than you expect.
    finish_state: Option<usize>,
}

impl Emulator {
    pub fn new(program: &[u8], load_addr: u16, start_addr: u16, breakpoints: Vec<u16>) -> Self {
        let mut mem = AddressSpace::new(&program, load_addr);
        let pc = mem.set_softev(start_addr);

        Self {
            cpu: Cpu::new(pc),
            mem,
            halted: false,
            num_instructions_executed: 0,
            breakpoints,
            finish_state: None,
        }
    }

    /// Called at ~300 Hz.
    pub fn sim_1000_instrs(&mut self) {
        for _ in 0..1_000 {
            self.step();
        }
    }

    fn step(&mut self) {
        if self.halted {
            return;
        }

        if self.check_breakpoints().is_break() {
            self.halted = true;

            eprintln!("{}", self.cpu.dbg_next_instr(&mut self.mem));
            eprint!("... ");

            return;
        }

        self.cpu.step(&mut self.mem);
        self.num_instructions_executed += 1;
    }

    fn check_breakpoints(&mut self) -> ControlFlow<()> {
        if self.cpu.next_instr(&mut self.mem).0 == Instr::Brk {
            eprintln!("\nwould break");
            return ControlFlow::Break(());
        }

        if self.cpu.would_halt(&mut self.mem) {
            eprintln!("\nwould halt");
            return ControlFlow::Break(());
        }

        if self.breakpoints.contains(&self.cpu.pc()) {
            eprintln!("\nhit breakpoint");
            return ControlFlow::Break(());
        }

        if let Some(depth) = self.finish_state.as_mut() {
            let next_instr = self.cpu.next_instr(&mut self.mem);
            match next_instr.0 {
                Instr::Jsr => *depth += 1,
                Instr::Rts => {
                    if *depth == 0 {
                        self.cpu.step(&mut self.mem);
                        self.num_instructions_executed += 1;

                        eprintln!("\nfinished subroutine");
                        self.finish_state = None;
                        return ControlFlow::Break(());
                    } else {
                        *depth -= 1;
                    }
                }
                _ => (),
            }
        }

        ControlFlow::Continue(())
    }

    /// Called at 60 Hz.
    pub fn draw_screen(&self) -> Vec<Vec<Color>> {
        self.mem.display()
    }

    pub fn key_down(&mut self, ascii_code: u8) {
        self.mem.key_down(ascii_code);
    }

    pub fn all_keys_up(&mut self) {
        self.mem.all_keys_up();
    }

    /// Execute a "debugger" command.
    pub fn control(&mut self, cmd: Command) {
        cmd.execute(self);
    }
}
