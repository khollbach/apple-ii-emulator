#![allow(unused_imports)] // todo

use std::{
    env,
    error::Error,
    fs::File,
    io::prelude::*,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{Context as _, Result};
use apple_ii_emulator::{
    cpu::{debugger::Debugger, Cpu},
    display::{gr, hgr, text},
    memory::Memory,
};
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
    let scale = 1; // todo: support non-trivial scaling (low prio for now)
    PhysicalSize::new(hgr::W as u32 * scale, hgr::H as u32 * scale)
};

fn main() -> Result<()> {
    let (filename,) = env::args()
        .skip(1)
        .collect_tuple()
        .context("expected 1 argument")?;
    let mut file = File::open(&filename)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    // todo: accept CLI args: --load-addr, --start-addr
    let mut mem = Memory::load_program(&prog, 0x2000); // load addr
    let pc = mem.set_softev(0x2000); // start addr

    let cpu = Arc::new(Mutex::new(Cpu::new(mem, pc)));
    let mut debugger = Debugger::new(Arc::clone(&cpu));

    thread::spawn(move || loop {
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
        // let dots = gr::ram_to_dots(&cpu.ram);
        // let dots = text::ram_to_dots(&cpu.ram);
        let dots = hgr::ram_to_dots_color(&cpu.mem.ram);

        let surface = self.surface.as_mut().unwrap();
        surface.resize(
            DESIRED_WINDOW_SIZE.width.try_into().unwrap(),
            DESIRED_WINDOW_SIZE.height.try_into().unwrap(),
        )?;

        let mut buf = surface.buffer_mut()?;
        for y in 0..hgr::H {
            for x in 0..hgr::W {
                let rgb = dots[y][x].rgb();
                buf[y * hgr::W + x] = pack_rgb(rgb);
            }
        }

        self.window.as_ref().unwrap().pre_present_notify();
        buf.present()?;

        Ok(())
    }
}

fn pack_rgb([r, g, b]: [u8; 3]) -> u32 {
    let r = r as u32;
    let g = g as u32;
    let b = b as u32;
    r << 16 | g << 8 | b
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
