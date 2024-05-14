use std::{collections::HashSet, io};

use anyhow::{bail, ensure, Context, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;

fn main() -> Result<()> {
    println!("match opcode {{");
    for l in io::stdin().lines() {
        let l = l?;
        let (code, instr, mode) = l
            .split_whitespace()
            .collect_tuple()
            .context("expected 3 fields")?;
        let code: u8 = code.parse().context("failed to parse code")?;
        let instr = format_instr(instr)?;
        let mode = format_mode(mode)?;
        println!("    {code:#04x} => ({instr}, {mode}),")
    }
    println!("}}");
    Ok(())
}

const INSTRS: [&str; 56] = [
    "adc", "and", "asl", "bcc", "bcs", "beq", "bit", "bmi", "bne", "bpl", "brk", "bvc", "bvs",
    "clc", "cld", "cli", "clv", "cmp", "cpx", "cpy", "dec", "dex", "dey", "eor", "inc", "inx",
    "iny", "jmp", "jsr", "lda", "ldx", "ldy", "lsr", "nop", "ora", "pha", "php", "pla", "plp",
    "rol", "ror", "rti", "rts", "sbc", "sec", "sed", "sei", "sta", "stx", "sty", "tax", "tay",
    "tsx", "txa", "txs", "tya",
];

fn format_instr(s: &str) -> Result<String> {
    static INSTRS_SET: Lazy<HashSet<String>> =
        Lazy::new(|| INSTRS.into_iter().map(String::from).collect());

    let mut s = s.to_lowercase();
    ensure!(INSTRS_SET.contains(&s), "unrecognized instr: {s:?}");

    assert!(s.is_ascii());
    unsafe {
        let first = &mut s.as_bytes_mut()[0];
        *first = first.to_ascii_uppercase();
    }

    Ok(format!("Instr::{s}"))
}

const MODES: [(&str, &str); 13] = [
    ("#", "Immediate"),
    ("A", "Accumulator"),
    ("abs", "Absolute"),
    ("abs,X", "AbsoluteX"),
    ("abs,Y", "AbsoluteY"),
    ("impl", "Implied"),
    ("ind", "Indirect"),
    ("ind,Y", "IndirectY"),
    ("rel", "Relative"),
    ("X,ind", "XIndirect"),
    ("zpg", "ZeroPage"),
    ("zpg,X", "ZeroPageX"),
    ("zpg,Y", "ZeroPageY"),
];

fn format_mode(s: &str) -> Result<String> {
    for (short_form, long_form) in MODES {
        if s == short_form {
            return Ok(format!("Mode::{long_form}"));
        }
    }
    bail!("invalid mode {s:?}")
}
