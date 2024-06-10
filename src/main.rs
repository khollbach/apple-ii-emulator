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
use apple_ii_emulator::{
    cpu::{Cpu, Debugger},
    hex,
    memory::Memory,
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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = File::open(&args.program)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    let mut mem = Memory::new(&prog, hex::decode_u16(&args.load_addr)?);
    let pc = mem.set_softev(hex::decode_u16(&args.start_addr)?);

    // These locks feels slightly kludgy, but I guess we'll keep it this way for
    // now:
    // * `cpu` is shared between: cpu thread, debugger thread
    // * `mem` is shared between all 3 threads: cpu, debugger, ui
    // * The cpu-thread and debbuger-thread only grab `mem` once they've
    //   already locked `cpu`. This is important to avoid deadlock.
    let shared_cpu = Arc::new(Mutex::new(Debugger::new(pc)));
    let shared_mem = Arc::new(Mutex::new(mem));
    let shared_halt = Arc::new(AtomicBool::new(false));

    let cpu = Arc::clone(&shared_cpu);
    let mem = Arc::clone(&shared_mem);
    let halt = Arc::clone(&shared_halt);
    thread::spawn(move || run_cpu(cpu, mem, halt));

    let cpu = Arc::clone(&shared_cpu);
    let mem = Arc::clone(&shared_mem);
    let halt = Arc::clone(&shared_halt);
    thread::spawn(move || match run_debugger(cpu, mem, halt) {
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

    event_loop.run_app(&mut WinitGui::new(shared_mem))?;

    Ok(())
}

fn run_cpu(cpu: Arc<Mutex<Debugger>>, mem: Arc<Mutex<Memory>>, halt: Arc<AtomicBool>) {
    loop {
        // hack: since 1 cycle != 1 instr, let's slow down a bit
        // Could look into cycle-accuracy at some point maybe (low-prio)
        // thread::sleep(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(3));

        if halt.load(Relaxed) {
            continue;
        }

        let mut cpu = cpu.lock().unwrap();
        let mut mem = mem.lock().unwrap();
        for _ in 0..1_000 {
            if cpu.step(&mut *mem).is_break() {
                halt.store(true, Relaxed);
            }
        }
    }
}

fn run_debugger(
    cpu: Arc<Mutex<Debugger>>,
    mem: Arc<Mutex<Memory>>,
    halt: Arc<AtomicBool>,
) -> Result<()> {
    let mut lines = io::stdin().lines();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let line = lines.next().context("EOF on stdin")??;

        let cmd = match Command::parse(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let mut cpu = cpu.lock().unwrap();

        let mut should_halt = true;
        match cmd {
            Command::Halt => (),
            Command::Continue => should_halt = false,
            Command::Step => {
                cpu.step(&mut *mem.lock().unwrap());
            }
        }
        halt.store(should_halt, Relaxed);
        // todo: this synchronization feels kinda messy.
        // Couldn't we just have a bunch of global state wrapped up in a single lock?

        if should_halt {
            eprintln!("{:?}", cpu.cpu);
        }
    }
}

/// CLI debugger command.
#[derive(Debug)]
enum Command {
    Halt,
    Continue,
    Step,
    // Break,
}

impl Command {
    fn parse(mut line: &str) -> Result<Command> {
        line = line.trim();

        let cmd = match line {
            "h" | "halt" => Command::Halt,
            "c" | "continue" => Command::Continue,
            // Default to "step", if the user just presses enter.
            "" | "s" | "step" => Command::Step,
            _ => bail!("invalid command: {line:?}"),
        };

        Ok(cmd)
    }
}
