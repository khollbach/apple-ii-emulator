use std::str::FromStr;

use anyhow::{bail, Context, Result};
use itertools::Itertools;

use crate::hex;

/// CLI debugger command.
#[derive(Debug)]
pub enum Command {
    Halt,
    Continue,
    Step,

    ToggleBreakpoint { addr: u16 },

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
            "s" | "step" => return Ok(Command::Step),
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
