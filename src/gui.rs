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
use itertools::Itertools;
use softbuffer::{Context, SoftBufferError, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopClosed, OwnedDisplayHandle},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::{
    cpu::Cpu,
    display::{color::Color, gr, hgr, text},
    memory::AddressSpace,
    Emulator,
};

/// What is the side-length (in physical pixels) of an emulated pixel (i.e. a
/// "dot of light" on the CRT display).
const SCALE: usize = 2;

const DESIRED_WINDOW_SIZE: PhysicalSize<u32> =
    PhysicalSize::new((hgr::W * SCALE) as u32, (hgr::H * SCALE) as u32);

type StdResult<T, E> = std::result::Result<T, E>;

/// Winit app.
pub struct Gui {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    occluded: bool,
    window_size: PhysicalSize<u32>,
    emu: Arc<Mutex<Emulator>>,
}

impl Gui {
    pub fn new(emu: Arc<Mutex<Emulator>>) -> Self {
        Self {
            window: None,
            surface: None,
            occluded: false,
            window_size: DESIRED_WINDOW_SIZE,
            emu,
        }
    }
}

impl ApplicationHandler for Gui {
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

impl Gui {
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> StdResult<(), Box<dyn Error>> {
        assert!(self.window.is_none());

        let attrs = Window::default_attributes()
            .with_inner_size(DESIRED_WINDOW_SIZE)
            .with_min_inner_size(DESIRED_WINDOW_SIZE)
            .with_max_inner_size(DESIRED_WINDOW_SIZE)
            .with_resizable(false);
        let window = Rc::new(event_loop.create_window(attrs)?);

        // Temporary hack, while debugging...
        if event_loop.available_monitors().count() >= 2 {
            window.set_outer_position(PhysicalPosition {
                x: 1920 + 100,
                y: 100,
            });
        }

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

            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => self.key_event(event),

            _ => (),
        }

        Ok(())
    }

    fn key_event(&self, e: KeyEvent) {
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

        if e.state.is_pressed() {
            self.emu.lock().unwrap().key_down(ascii_code);
        } else {
            // todo: this is technically not a correct implementation, but I'm
            // guessing it'll feel "good enough", for now.
            self.emu.lock().unwrap().all_keys_up();
        }
    }

    fn redraw(&mut self) -> StdResult<(), SoftBufferError> {
        let dots = self.emu.lock().unwrap().draw_screen();

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
