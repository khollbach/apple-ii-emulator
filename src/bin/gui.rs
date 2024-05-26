use std::{error::Error, rc::Rc, thread, time::Duration};

use anyhow::Result;
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
    let event_loop = EventLoop::new()?;

    // Step the CPU every 1 microsecond. In other words, run at 1MHz.
    let event_tx = event_loop.create_proxy();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_micros(1));
        match event_tx.send_event(()) {
            Ok(()) => (),
            Err(EventLoopClosed(())) => return,
        }
    });

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    occluded: bool,
    window_size: PhysicalSize<u32>,
    toy_state: bool,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            occluded: false,
            window_size: DESIRED_WINDOW_SIZE,
            toy_state: false,
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
        let window = self.window.as_mut().unwrap();

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
                device_id: _,
                event: _,
                is_synthetic: _,
            } => {
                self.toy_state ^= true;
                window.request_redraw();
            }

            _ => (),
        }

        Ok(())
    }

    fn redraw(&mut self) -> StdResult<(), SoftBufferError> {
        let surface = self.surface.as_mut().unwrap();

        let one = 1.try_into().unwrap();
        surface.resize(
            self.window_size.width.try_into().unwrap_or(one),
            self.window_size.height.try_into().unwrap_or(one),
        )?;

        let mut buf = surface.buffer_mut()?;
        if self.toy_state {
            buf.fill(0xffff_ffff);
        } else {
            buf.fill(0);
        };

        self.window.as_mut().unwrap().pre_present_notify();
        buf.present()?;

        Ok(())
    }

    fn tick(&mut self) {
        // todo
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
        self.tick()
    }
}
