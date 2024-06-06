//! This is pretty hacked-together, but I'll probably only make improvements
//! when needed.

use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::{
    cpu::{opcode, Cpu},
    hex,
};

pub struct Debugger {
    cpu: Arc<Mutex<Cpu>>,
    num_instructions_executed: u64,
    breakpoints: Vec<u16>,
    single_step: bool,
}

impl Debugger {
    pub fn new(cpu: Arc<Mutex<Cpu>>) -> Self {
        Self {
            cpu,
            num_instructions_executed: 0,
            // add breakpoints here as needed
            breakpoints: vec![],
            single_step: false,
        }
    }

    pub fn run(mut self) -> ! {
        loop {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let mut cpu = self.cpu.lock().unwrap();

        if self.breakpoints.contains(&cpu.pc) {
            self.single_step = true;
        }

        if cpu.would_halt() {
            eprintln!("would halt");
            self.single_step = true;
        }

        // Detect long-running loops that aren't a simple "halt instruction".
        let i = self.num_instructions_executed;
        if i != 0 && i % 100_000_000 == 0 {
            eprintln!("after {}M instructions,", i / 1_000_000);
            eprintln!("{:?}", cpu);
            eprintln!();
        }

        if self.single_step {
            eprintln!("{:?}", cpu);
            let (instr, mode) = opcode::decode(cpu.ram[cpu.pc as usize]);
            let instr_bytes = &cpu.ram[cpu.pc as usize..][..mode.instr_len() as usize];
            eprintln!("next instr: {:02x?} {:?} {:?}", instr_bytes, instr, mode);

            loop {
                // don't hold the lock on the cpu while we're getting input!
                drop(cpu);
                let line: String = io::stdin().lines().next().unwrap().unwrap();
                cpu = self.cpu.lock().unwrap();

                let cmd = line.trim();
                if cmd.is_empty() {
                    break;
                }

                if cmd.contains('.') {
                    eprintln!("not yet implemented: range of bytes");
                    continue;
                }

                let is_valid = cmd.chars().all(|c| c.is_digit(16)) && cmd.len() <= 4;
                if !is_valid {
                    continue;
                }

                let addr = hex::decode_u16(cmd).unwrap();
                eprintln!("ram[${:04x}]: ${:02x}", addr, cpu.get_byte(addr));
            }
        }

        cpu.step();
        self.num_instructions_executed += 1;
    }
}
