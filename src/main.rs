#![allow(unused_imports)] // todo

use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use anyhow::{bail, Context as _, Result};
use apple_ii_emulator::{debugger_commands::Command, gui::Gui, hex, Emulator};
use clap::{command, Parser};
use itertools::Itertools;
use winit::event_loop::{EventLoop, EventLoopClosed};

/// Apple IIe emulator (work in progress!)
#[derive(Parser)]
struct Args {
    /// The memory image file format is that of llvm-mos. From their docs:
    ///
    /// The image file is a collection of blocks. Each block consists of a
    /// 16-bit starting address, then a 16-bit block size, then that many bytes
    /// of contents. Both the address and size are stored little-endian.
    memory_image_file: String,

    /// Use this if your input file is just binary machine code -- no headers or
    /// anything.
    ///
    /// Provide a memory address, in hexadecimal. We load your code into memory
    /// at this offset, and then jump to it.
    #[arg(long, value_name = "START_ADDR")]
    raw_bytes: Option<String>,

    /// Memory address (hexadecimal) to set a breakpoint in the debugger. Can be
    /// passed multiple times.
    #[arg(long)]
    breakpoint: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut breakpoints = Vec::with_capacity(args.breakpoint.len());
    for bp in args.breakpoint {
        let addr = hex::decode_u16(&bp)?;
        breakpoints.push(addr);
    }

    let emu = if let Some(load_addr) = args.raw_bytes {
        let load_addr = hex::decode_u16(&load_addr)?;
        let start_addr = load_addr;

        let mut file = File::open(&args.memory_image_file)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        // Treat the file as raw bytes.
        Emulator::new(&bytes, load_addr, start_addr, breakpoints)
    } else {
        let mut file = File::open(&args.memory_image_file)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        // Read the file headers.
        Emulator::from_memory_image(&bytes, breakpoints)?
    };
    let emu = Arc::new(Mutex::new(emu));

    let emu1 = Arc::clone(&emu);
    thread::spawn(move || run_cpu(emu1));

    let emu1 = Arc::clone(&emu);
    thread::spawn(move || match run_debugger(emu1) {
        Ok(()) => (),
        Err(e) => eprintln!("\n{e}"),
    });

    // Re-draw the screen at 60 Hz. This isn't the "right" way to do it, but
    // it's probably fine for now. See the winit docs for more ideas.
    let event_loop = EventLoop::new()?;
    let event_tx = event_loop.create_proxy();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs_f64(1. / 60.));
        match event_tx.send_event(()) {
            Ok(()) => (),
            Err(EventLoopClosed(())) => return,
        }
    });

    let mut gui = Gui::new(emu);
    event_loop.run_app(&mut gui)?;

    Ok(())
}

fn run_cpu(emu: Arc<Mutex<Emulator>>) {
    loop {
        // hack: since 1 cycle != 1 instr, let's slow down a bit
        // Could look into cycle-accuracy at some point maybe (low-prio)
        //thread::sleep(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(3));

        emu.lock().unwrap().sim_1000_instrs();
    }
}

fn run_debugger(emu: Arc<Mutex<Emulator>>) -> Result<()> {
    let mut lines = io::stdin().lines();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let line = lines.next().context("EOF on stdin")??;

        match parse_line(&line) {
            Ok(cmd) => emu.lock().unwrap().control(cmd),
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }
}

fn parse_line(mut line: &str) -> Result<Command> {
    line = line.trim();
    if line.is_empty() {
        return Ok(Command::Step);
    }
    line.parse()
}
