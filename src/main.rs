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
use apple_ii_emulator::{commands::Command, gui::Gui, hex, Emulator};
use clap::{command, Parser};
use itertools::Itertools;
use winit::event_loop::{EventLoop, EventLoopClosed};

/// Apple IIe emulator (work in progress!)
#[derive(Parser)]
#[command()]
struct Args {
    /// File name of a 6502 program -- just binary machine code, no file headers or anything.
    #[arg()]
    program: String,

    /// Memory address to load the program at.
    #[arg(long)]
    load_addr: String,

    /// Memory address of the first instruction to execute.
    #[arg(long)]
    start_addr: String,

    /// Memory address to set a breakpoint in the debugger. Can be passed
    /// multiple times.
    #[arg(long)]
    breakpoint: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let load_addr = hex::decode_u16(&args.load_addr)?;
    let start_addr = hex::decode_u16(&args.start_addr)?;

    let mut breakpoints = Vec::with_capacity(args.breakpoint.len());
    for bp in args.breakpoint {
        let addr = hex::decode_u16(&bp)?;
        breakpoints.push(addr);
    }

    let mut file = File::open(&args.program)?;
    let mut program = vec![];
    file.read_to_end(&mut program)?;

    let emu = Arc::new(Mutex::new(Emulator::new(
        &program,
        load_addr,
        start_addr,
        breakpoints,
    )));

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
