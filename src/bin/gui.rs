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
use apple_ii_emulator::{
    cpu::{debugger::Debugger, Cpu, MEM_LEN},
    display::{self, color::Color, gr, hgr, text},
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
    let mut ram = load_program(&prog, 0x2000); // load addr

    // temporary hack: run RESET routine, but skip disk loading code.
    // This sets up required state for keyboard input routines, etc.
    // Once we figure out how interrupts work, we can re-visit this.
    assert_eq!(ram[0x03f2..][..3], [0, 0, 0]);
    ram[0x03f2] = 0x00; // start_addr lo
    ram[0x03f3] = 0x20; // start_addr hi
    ram[0x03f4] = 0xa5 ^ ram[0x03f3]; // magic number to indicate "warm start"
    let pc = 0xfa62; // RESET

    let cpu = Arc::new(Mutex::new(Cpu::new(ram, pc)));
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

fn load_program(prog: &[u8], load_addr: u16) -> Vec<u8> {
    struct Slice<'a> {
        offset: usize,
        bytes: &'a [u8],
    }

    let mut slices = vec![];
    slices.push(Slice {
        bytes: prog,
        offset: load_addr.into(),
    });

    let rom_f8 = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_F8ROM"));
    debug_assert_eq!(rom_f8.len(), 0x800);
    slices.push(Slice {
        bytes: rom_f8,
        offset: 0xf800,
    });

    let rom_applesoft = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Applesoft"));
    debug_assert_eq!(rom_applesoft.len(), 0x2800);
    slices.push(Slice {
        bytes: rom_applesoft,
        offset: 0xd000,
    });

    let rom_80col = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_80col"));
    debug_assert_eq!(rom_80col.len(), 0x300 + 0x800 - 1);
    slices.push(Slice {
        bytes: &rom_80col[..0x300],
        offset: 0xc100,
    });
    slices.push(Slice {
        bytes: &rom_80col[0x300..],
        offset: 0xc800,
    });

    // Check the slices don't overlap.
    slices.sort_by_key(|s| s.offset);
    for w in slices.windows(2) {
        let [s1, s2] = w else { unreachable!() };
        assert!(s1.offset + s1.bytes.len() <= s2.offset);
    }

    let mut ram = vec![0; MEM_LEN];
    for s in slices {
        ram[s.offset..][..s.bytes.len()].copy_from_slice(s.bytes);
    }
    ram
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
        let dots = hgr::ram_to_dots_color(&cpu.ram);

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
