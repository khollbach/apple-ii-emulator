#![allow(unused_imports)] // todo

use std::{
    env,
    error::Error,
    fs::File,
    io::prelude::*,
    ops::ControlFlow,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context as _, Result};
use apple_ii_emulator::cpu::{debugger::Debugger, Cpu, MEM_LEN};
use itertools::Itertools;
use softbuffer::{Context, SoftBufferError, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, OwnedDisplayHandle},
    window::{Window, WindowId},
};

type StdResult<T, E> = std::result::Result<T, E>;

const DESIRED_WINDOW_SIZE: PhysicalSize<u32> = {
    let scale = 2;
    PhysicalSize::new(280 * scale, 192 * scale)
};

fn main() -> Result<()> {
    // todo: accept CLI args: --load-addr --start-addr

    let (filename,) = env::args()
        .skip(1)
        .collect_tuple()
        .context("expected 1 argument")?;
    let mut file = File::open(&filename)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    let mut ram = vec![0; MEM_LEN];
    ram[0x2000..][..prog.len()].copy_from_slice(&prog);

    let cpu = Arc::new(Mutex::new(Cpu::new(ram, 0x2000)));
    let mut debugger = Debugger::new(Arc::clone(&cpu));

    thread::spawn(move || {
        // hack: since 1 cycle != 1 instr, let's slow down a bit
        // Could look into cycle-accuracy at some point maybe (low-prio)
        // thread::sleep(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(3));

        for _ in 0..1_000 {
            debugger.step();
        }
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

    event_loop.run_app(&mut App::new(cpu))?;

    Ok(())
}

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    occluded: bool,
    window_size: PhysicalSize<u32>,
    cpu: Arc<Mutex<Cpu>>,
}

impl App {
    fn new(cpu: Arc<Mutex<Cpu>>) -> Self {
        Self {
            window: None,
            surface: None,
            occluded: false,
            window_size: DESIRED_WINDOW_SIZE,
            cpu,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> StdResult<(), Box<dyn Error>> {
        assert!(self.window.is_none());

        let attrs = Window::default_attributes()
            .with_inner_size(DESIRED_WINDOW_SIZE)
            .with_min_inner_size(DESIRED_WINDOW_SIZE)
            .with_max_inner_size(DESIRED_WINDOW_SIZE)
            .with_resizable(false);
        let window = Rc::new(event_loop.create_window(attrs)?);
        self.window = Some(Rc::clone(&window));

        let context = Context::new(event_loop.owned_display_handle())?;
        self.surface = Some(Surface::new(&context, window)?);

        Ok(())
    }

    fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
    ) -> StdResult<(), SoftBufferError> {
        let window = self.window.as_ref().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Occluded(occluded) => {
                if self.occluded != occluded {
                    self.occluded = occluded;
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested if !self.occluded => self.redraw()?,

            WindowEvent::Resized(mut size) => {
                if size != DESIRED_WINDOW_SIZE {
                    if let Some(actual) = window.request_inner_size(DESIRED_WINDOW_SIZE) {
                        size = actual;
                    }
                }

                if self.window_size != size {
                    self.window_size = size;
                    window.request_redraw();
                }
            }

            _ => (),
        }

        Ok(())
    }

    fn redraw(&mut self) -> StdResult<(), SoftBufferError> {
        // We probably don't need to clone the whole 64KiB ram, but this seems
        // fine for now.
        let cpu = self.cpu.lock().unwrap().clone();
        // eprintln!("\n{:?}", cpu);

        // TODO:
        // * debug test prog not blinking the screen any more
        // * then commit what we have so far and start thinking about the wobbly
        //      tunnel demo / GR display mode
        // * maybe could impl a text-based dump of screen memory as a starting
        //      point, before going to actual graphics

        let surface = self.surface.as_mut().unwrap();

        let one = 1.try_into().unwrap();
        surface.resize(
            self.window_size.width.try_into().unwrap_or(one),
            self.window_size.height.try_into().unwrap_or(one),
        )?;

        // For now, paint the entire screen, indicating whether the byte at $400
        // is non-zero.
        let mut buf = surface.buffer_mut()?;
        let color: u32 = if cpu.ram[0x400] != 0 { 0x_00ff_ffff } else { 0 };
        buf.fill(color);

        self.window.as_ref().unwrap().pre_present_notify();
        buf.present()?;

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_window(event_loop).unwrap();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_window_event(event_loop, event).unwrap();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
