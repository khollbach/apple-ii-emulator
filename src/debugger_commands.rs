use std::str::FromStr;

use anyhow::{bail, ensure, Context, Result};
use itertools::Itertools;

use crate::{hex, memory::AddressSpace, Emulator};

/// CLI debugger command.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    Halt,
    Continue,
    CpuInfo,

    Step,
    ToggleBreakpoint { addr: u16 },
    Finish,

    ShowByte { addr: u16 },
    ShowRange { start: u16, end_inclusive: u16 },
    // other ideas for commands:
    // * goto (set pc)
    // * jsr (which auto-breaks when we return all the way back)
    // * step `n` times
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(mut s: &str) -> Result<Self> {
        s = s.trim();

        match s {
            "h" | "halt" => return Ok(Command::Halt),
            "c" | "continue" => return Ok(Command::Continue),
            "i" | "info" => return Ok(Command::CpuInfo),
            "s" | "step" => return Ok(Command::Step),
            "f" | "finish" => return Ok(Command::Finish),
            _ => (),
        }

        let mut words = s.split_whitespace();
        let first = words.next().context("empty command")?;
        if matches!(first, "b" | "break") {
            let (addr,) = words
                .collect_tuple()
                .context("expected 1 argument to break")?;
            let addr = hex::decode_u16(addr)?;
            return Ok(Command::ToggleBreakpoint { addr });
        }

        if s.contains('.') {
            let (start, end) = s.split_once('.').unwrap();
            let start = hex::decode_u16(start)?;
            let end_inclusive = hex::decode_u16(end)?;
            ensure!(start <= end_inclusive);
            return Ok(Command::ShowRange {
                start,
                end_inclusive,
            });
        }

        if let Ok(addr) = hex::decode_u16(s) {
            return Ok(Command::ShowByte { addr });
        }

        bail!("invalid command: {s:?}");
    }
}

impl Command {
    pub fn execute(self, emu: &mut Emulator) {
        match self {
            Command::Halt => {
                if emu.halted {
                    println!("already halted");
                } else {
                    emu.halted = true;
                    println!("{}", emu.cpu.dbg_next_instr(&mut emu.mem));
                }
            }
            Command::Continue => {
                if !emu.halted {
                    println!("already running");
                } else {
                    emu.halted = false;

                    // Skip past the current breakpoint. (Instead of breaking
                    // right away and going nowhere.)
                    if emu.breakpoints.contains(&emu.cpu.pc()) {
                        emu.cpu.step(&mut emu.mem);
                        emu.num_instructions_executed += 1;
                    }
                }
            }
            Command::CpuInfo => {
                println!("{:?}", emu.cpu);
            }

            Command::Step => {
                if !emu.halted {
                    println!("halting");
                    emu.halted = true;
                }
                emu.cpu.step(&mut emu.mem);
                emu.num_instructions_executed += 1;

                println!("{}", emu.cpu.dbg_next_instr(&mut emu.mem));
            }

            Command::ToggleBreakpoint { addr } => {
                if let Some((idx, _)) = emu.breakpoints.iter().find_position(|&&a| a == addr) {
                    emu.breakpoints.swap_remove(idx);
                    println!("cleared breakpoint ${:04x}", addr);
                } else {
                    emu.breakpoints.push(addr);
                    println!("set breakpoint ${:04x}", addr);
                }
            }

            Command::Finish => {
                if !emu.halted {
                    println!("already running; please halt first");
                    return;
                }

                if let Some(d) = emu.finish_state {
                    println!("warning already trying to finish a function, current depth: {d}");
                    println!("overriding...");
                }

                emu.halted = false;
                emu.finish_state = Some(0);
            }

            Command::ShowByte { addr } => {
                println!("ram[${:04x}]: ${:02x}", addr, emu.mem.read(addr));
            }
            Command::ShowRange {
                start,
                end_inclusive,
            } => show_range(&mut emu.mem, start, end_inclusive),
        }
    }
}

fn show_range(mem: &mut AddressSpace, start: u16, end_inclusive: u16) {
    let start_rounded_down = start / 16 * 16;

    for addr in start_rounded_down..=end_inclusive {
        if addr != start_rounded_down {
            match addr % 16 {
                0 => println!(),
                8 => print!("  "),
                _ => print!(" "),
            }

            // Blank line at half-page mark.
            if addr % 256 == 128 {
                println!();
            }
            // Second blank line between pages.
            if addr % 256 == 0 {
                println!();
            }
        }

        if addr >= start {
            print!("{:02x}", mem.read(addr));
        } else {
            print!("  ");
        }
    }

    println!();
}
