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
    cpu::{Cpu, Debugger},
    display::{color::Color, gr, hgr, text},
    memory::{Mem, Memory},
};
use itertools::Itertools;
use softbuffer::{Context, SoftBufferError, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, OwnedDisplayHandle},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

type StdResult<T, E> = std::result::Result<T, E>;

/// What is the side-length (in physical pixels) of an emulated pixel (i.e. a
/// "dot of light" on the CRT display).
const SCALE: usize = 2;

const DESIRED_WINDOW_SIZE: PhysicalSize<u32> =
    PhysicalSize::new(hgr::W as u32 * SCALE as u32, hgr::H as u32 * SCALE as u32);

fn main() -> Result<()> {
    let (filename,) = env::args()
        .skip(1)
        .collect_tuple()
        .context("expected 1 argument")?;
    let mut file = File::open(&filename)?;

    let mut prog = vec![];
    file.read_to_end(&mut prog)?;

    // todo: accept CLI args: --load-addr, --start-addr
    let mut mem = Mem::new(&prog, 0x2000); // load addr
    let pc = mem.set_softev(0x2000); // start addr

    let shared_mem = Arc::new(Mutex::new(mem));

    let mem = Arc::clone(&shared_mem);
    thread::spawn(move || {
        let mut cpu = Debugger::new(pc);
        loop {
            // hack: since 1 cycle != 1 instr, let's slow down a bit
            // Could look into cycle-accuracy at some point maybe (low-prio)
            // thread::sleep(Duration::from_millis(1));
            thread::sleep(Duration::from_millis(3));

            for _ in 0..1_000 {
                cpu.step(&mem);
            }
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

    event_loop.run_app(&mut App::new(shared_mem))?;

    Ok(())
}

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    occluded: bool,
    window_size: PhysicalSize<u32>,
    mem: Arc<Mutex<Mem>>,
}

impl App {
    fn new(mem: Arc<Mutex<Mem>>) -> Self {
        Self {
            window: None,
            surface: None,
            occluded: false,
            window_size: DESIRED_WINDOW_SIZE,
            mem,
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

    fn window_event(
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

            WindowEvent::KeyboardInput { event, .. } => self.key_event(event),

            _ => (),
        }

        Ok(())
    }

    fn key_event(&self, e: KeyEvent) {
        if !e.state.is_pressed() {
            return;
        }

        // This mapping probably isn't 100% accurate, and we aren't handling
        // modifiers very carefully. See the table on page 13 of the //e
        // Technical Reference Manual for more ideas.
        let ascii_code = match e.logical_key {
            Key::Named(key) => {
                match key {
                    NamedKey::Backspace => 0x7f,
                    NamedKey::ArrowLeft => 0x08,
                    NamedKey::Tab => 0x09,
                    NamedKey::ArrowDown => 0x0a,
                    NamedKey::ArrowUp => 0x0b,
                    NamedKey::Enter => 0x0d,
                    NamedKey::ArrowRight => 0x15,
                    NamedKey::Escape => 0x1b,
                    NamedKey::Space => 0x20,
                    NamedKey::Insert => todo!(), // (low-prio todo: use Insert as RESET?)
                    _ => return,
                }
            }
            Key::Character(key) if key.len() == 1 => key.as_bytes()[0],
            _ => return,
        };

        self.mem.lock().unwrap().key_down(ascii_code);
    }

    fn redraw(&mut self) -> StdResult<(), SoftBufferError> {
        // We probably don't need to clone the whole 64KiB ram, but this seems
        // fine for now.
        let mem = self.mem.lock().unwrap().clone();

        // TODO at some point: render all 3 screens, for easier debugging.
        // let dots = gr::dots(mem.gr_page1());
        let dots = text::dots(mem.gr_page1());
        // let dots = hgr::dots_color(mem.hgr_page1());

        let surface = self.surface.as_mut().unwrap();
        surface.resize(
            DESIRED_WINDOW_SIZE.width.try_into().unwrap(),
            DESIRED_WINDOW_SIZE.height.try_into().unwrap(),
        )?;

        let mut buf = surface.buffer_mut()?;
        paint_surface(&dots, &mut buf);

        self.window.as_ref().unwrap().pre_present_notify();
        buf.present()?;

        Ok(())
    }
}

fn paint_surface(dots: &Vec<Vec<Color>>, buf: &mut [u32]) {
    for y in 0..hgr::H {
        for x in 0..hgr::W {
            let rgb = dots[y][x].rgb();
            let pixel = pack_rgb(rgb);
            for i in 0..SCALE {
                let row = y * SCALE + i;
                let col = x;
                buf[(row * hgr::W + col) * SCALE..][..SCALE].fill(pixel);
            }
        }
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
        self.window_event(event_loop, event).unwrap();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
