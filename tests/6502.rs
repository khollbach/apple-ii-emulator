/*
use std::{env, fs::File, io::prelude::*};

use anyhow::{Context, Result};
use apple_ii_emulator::{cpu::Cpu, hex, memory::Memory};
use itertools::Itertools;

fn main() -> Result<()> {
    let load_addr = env::var("LOAD_ADDR").context("expected LOAD_ADDR=<hex-string>")?;
    let load_addr = hex::decode_u16(&load_addr)?;
    let start_addr = env::var("START_ADDR").context("expected START_ADDR=<hex-string>")?;
    let start_addr = hex::decode_u16(&start_addr)?;

    let (filename,) = env::args()
        .skip(1)
        .collect_tuple()
        .context("expected 1 argument: filename")?;
    let mut file = File::open(&filename)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    let mem = Memory::load_program(&prog, load_addr);
    Cpu::new(mem, start_addr).run();

    Ok(())
}
*/

// TODO: set this up as an automated test somehow.
