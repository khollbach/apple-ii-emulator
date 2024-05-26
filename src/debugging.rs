use std::{fmt, io};

use crate::{instr::Mode, opcode, Cpu};

impl Cpu {
    /// If the CPU would "halt" gracefully, this will return instead of looping.
    /// This can be useful for debugging.
    pub fn run_until_halt(mut self, start_addr: u16) -> Self {
        let mut enable_debugger = false;

        self.pc = start_addr;
        for i in 0.. {
            if self.would_halt() {
                eprintln!("would halt");
                enable_debugger = true;
            }

            // Detect long-running loops that aren't a simple "halt instruction".
            if i != 0 && i % 100_000_000 == 0 {
                eprintln!("after {}M instructions,", i / 1_000_000);
                eprintln!("{self:?}");
                eprintln!();
            }

            // // """breakpoint"""
            // if self.pc == 0x35e3 {
            //     enable_debugger = true;
            // }
            // // (currently halting at $35ef)

            // hacky "single-step debugger", for testing
            if enable_debugger {
                eprintln!("{:?}", self);
                let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);
                let instr_bytes = &self.ram[self.pc as usize..][..mode.instr_len() as usize];
                eprintln!("next instr: {:02x?} {:?} {:?}", instr_bytes, instr, mode);

                loop {
                    let line: String = io::stdin().lines().next().unwrap().unwrap();
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

                    let padding = "0".repeat(4_usize.saturating_sub(cmd.len()));
                    let cmd = format!("{}{}", padding, cmd);
                    let addr = hex::decode(cmd).unwrap();
                    assert_eq!(addr.len(), 2);
                    let addr = u16::from_be_bytes([addr[0], addr[1]]);
                    eprintln!("ram[${:04x}]: ${:02x}", addr, self.get_byte(addr));
                }
            }

            // hack: detect jumping to `start` label.
            let (_, mode) = opcode::decode(self.ram[self.pc as usize]);
            let instr_bytes = &self.ram[self.pc as usize..][..mode.instr_len() as usize];
            if instr_bytes == [0x4c, 0x00, 0x04] {
                eprintln!("would jump to start");
                break;
            }

            self.step();
        }
        self
    }

    /// Hack for testing: detect a "halt" instruction.
    fn would_halt(&self) -> bool {
        self.would_halt_jmp() || self.would_halt_branch()
    }

    fn would_halt_jmp(&self) -> bool {
        let [lo, hi] = self.pc.to_le_bytes();
        let jmp_absolute = 0x4c;
        let halt = [jmp_absolute, lo, hi];
        &self.ram[self.pc as usize..][..3] == &halt
    }

    fn would_halt_branch(&self) -> bool {
        let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);
        let is_branch = mode == Mode::Relative;
        let in_place = self.get_byte(self.pc.checked_add(1).unwrap()) as i8 == -2;
        is_branch && in_place && self.would_branch(instr)
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pc: ${:04x}", self.pc)?;
        writeln!(f, "sp: ${:02x}", self.sp)?;
        writeln!(f, "flags: {:08b}", self.flags.bits)?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "a: ${:02x}", self.a)?;
        writeln!(f, "x: ${:02x}", self.x)?;
        write!(f, "y: ${:02x}", self.y)?;
        Ok(())
    }
}
