#![allow(unused_imports)] // todo

use std::{
    env,
    fs::File,
    io::prelude::*,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context as _, Result};
use apple_ii_emulator::{
    cpu::{Cpu, Debugger},
    hex,
    memory::Mem,
    winit_gui::WinitGui,
};
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

    /// Where in memory you want your program to be loaded.
    #[arg(long, default_value = "$6000")]
    load_addr: String,

    /// Memory address of the first instruction to start executing.
    #[arg(long, default_value = "$6000")]
    start_addr: String,

    /// Disable debugger.
    #[arg(long)]
    no_debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(&args.program)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    let mut mem = Mem::new(&prog, hex::decode_u16(&args.load_addr)?);
    let pc = mem.set_softev(hex::decode_u16(&args.start_addr)?);

    let shared_mem = Arc::new(Mutex::new(mem));
    let mem = Arc::clone(&shared_mem);

    if args.no_debug {
        thread::spawn(move || {
            let mut cpu = Cpu::new(pc);
            loop {
                // hack: since 1 cycle != 1 instr, let's slow down a bit
                // Could look into cycle-accuracy at some point maybe (low-prio)
                // thread::sleep(Duration::from_millis(1));
                thread::sleep(Duration::from_millis(3));

                let mut mem = mem.lock().unwrap();
                for _ in 0..1_000 {
                    cpu.step(&mut *mem);
                }
            }
        });
    } else {
        todo!()
    }

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

    event_loop.run_app(&mut WinitGui::new(shared_mem))?;

    Ok(())
}
