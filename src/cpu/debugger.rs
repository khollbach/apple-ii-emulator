//! This is pretty hacked-together, but I'll probably only make improvements
//! when needed.

use std::{
    io,
    ops::{ControlFlow, DerefMut},
    sync::{Arc, Mutex},
};

use super::instr::Mode;
use crate::{
    cpu::{instr, Cpu},
    hex,
    memory::Memory,
};

pub struct Debugger {
    pub cpu: Cpu,
    num_instructions_executed: u64,
    breakpoints: Vec<u16>,
    // single_step: bool,
}

impl Debugger {
    pub fn new(start_addr: u16) -> Self {
        Self {
            cpu: Cpu::new(start_addr),
            num_instructions_executed: 0,
            // add breakpoints here as needed
            breakpoints: vec![],
            // single_step: false,
        }
    }

    pub fn step(&mut self, mem: &mut Memory) -> ControlFlow<()> {
        if self.breakpoints.contains(&self.cpu.pc) {
            return ControlFlow::Break(());
        }

        if would_halt(&self.cpu, &mut *mem) {
            eprintln!("would halt");
            return ControlFlow::Break(());
        }

        if mem.get(self.cpu.pc) == 0 {
            eprintln!("would break");
            return ControlFlow::Break(());
        }

        // Detect long-running loops that aren't a simple "halt instruction".
        let i = self.num_instructions_executed;
        if i != 0 && i % 100_000_000 == 0 {
            eprintln!("after {}M instructions,", i / 1_000_000);
            eprintln!("{:?}", self.cpu);
            eprintln!();
        }

        // if self.single_step {
        //     eprintln!("{:?}", self.cpu);
        //     let (instr, mode) = instr::decode(mem.get(self.cpu.pc));
        //     let instr_bytes = &curr_instr(&self.cpu, mem)[..mode.instr_len() as usize];
        //     eprintln!("next instr: {:02x?} {:?} {:?}", instr_bytes, instr, mode);

        //     loop {
        //         // don't hold the lock while we're getting input
        //         // drop(mem);
        //         let line: String = io::stdin().lines().next().unwrap().unwrap();
        //         // mem = shared_mem.lock().unwrap();

        //         let cmd = line.trim();
        //         if cmd.is_empty() {
        //             break;
        //         }

        //         if cmd.contains('.') {
        //             let (start, end) = cmd.split_once('.').unwrap();
        //             let start = hex::decode_u16(start).unwrap();
        //             let end = hex::decode_u16(end).unwrap();

        //             eprintln!("ram[${:04x}..=${:04x}]:", start, end);
        //             for addr in start..=end {
        //                 eprint!("${:02x} ", mem.get(addr));
        //             }
        //             eprintln!();
        //             continue;
        //         }

        //         let is_valid = cmd.chars().all(|c| c.is_digit(16)) && cmd.len() <= 4;
        //         if !is_valid {
        //             continue;
        //         }

        //         let addr = hex::decode_u16(cmd).unwrap();
        //         eprintln!("ram[${:04x}]: ${:02x}", addr, mem.get(addr));
        //     }
        // }

        self.cpu.step(mem);
        self.num_instructions_executed += 1;

        ControlFlow::Continue(())
    }
}

fn curr_instr(cpu: &Cpu, mem: &mut Memory) -> [u8; 3] {
    [mem.get(cpu.pc), mem.get(cpu.pc + 1), mem.get(cpu.pc + 2)]
}

/// Detect a "halt" instruction.
fn would_halt(cpu: &Cpu, mem: &mut Memory) -> bool {
    would_halt_jmp(cpu, mem) || would_halt_branch(cpu, mem)
}

fn would_halt_jmp(cpu: &Cpu, mem: &mut Memory) -> bool {
    let [lo, hi] = cpu.pc.to_le_bytes();
    let jmp_absolute = 0x4c;
    let halt = [jmp_absolute, lo, hi];
    curr_instr(cpu, mem) == halt
}

fn would_halt_branch(cpu: &Cpu, mem: &mut Memory) -> bool {
    let (instr, mode) = instr::decode(mem.get(cpu.pc));
    let is_branch = mode == Mode::Relative;
    let in_place = mem.get(cpu.pc + 1) as i8 == -2;
    is_branch && in_place && cpu.would_branch(instr)
}
