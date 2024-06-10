#![allow(unused_imports)] // todo

use cpu::Cpu;
use debugger::Command;
use display::{color::Color, gr, hgr, text};
use memory::Memory;

mod cpu;
mod debugger;
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
            // todo: detect BRK / breakpoint / halt
            if false || self.breakpoints.contains(&0_0_0) {
                // todo: print cpu info + a message describing the cause of halt
                self.halted = true;
                return;
            }

            self.cpu.step(&mut self.mem);
            self.num_instructions_executed += 1;
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
        unimplemented!();
    }

    pub fn control(&mut self, _cmd: Command) {
        unimplemented!();

        // let mut should_halt = true;
        // match cmd {
        //     Command::Halt => (),
        //     Command::Continue => should_halt = false,
        //     Command::Step => {
        //         cpu.step(&mut *mem.lock().unwrap());
        //         self.num_instructions_executed += 1;
        //     }
        // }
        // halt.store(should_halt, Relaxed);
        // // todo: this synchronization feels kinda messy.
        // // Couldn't we just have a bunch of global state wrapped up in a single lock?

        // if should_halt {
        //     eprintln!("{:?}", cpu.cpu);
        // }
    }
}
