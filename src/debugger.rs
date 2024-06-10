use std::str::FromStr;

use anyhow::{bail, Result};

/// CLI debugger command.
#[derive(Debug)]
pub enum Command {
    Halt,
    Continue,
    Step,
    // Break,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    /// Allows leading and trailing whitespace.
    fn from_str(mut s: &str) -> Result<Self> {
        s = s.trim();

        let cmd = match s {
            "h" | "halt" => Command::Halt,
            "c" | "continue" => Command::Continue,
            // Default to "step", if the user just presses enter.
            "" | "s" | "step" => Command::Step,
            _ => bail!("invalid command: {s:?}"),
        };

        Ok(cmd)
    }
}
