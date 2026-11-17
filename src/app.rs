use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::renderer::Render;

pub struct App{
    window: Option<Arc<Window>>,
    renderer: Option<Render>,
    time: f32,
    last_frame: Instant,
}

impl App{
    pub fn new()->Self{
        Self{
            window: None,
            renderer: None,
            time: 0.0,
            last_frame: Instant::now(),
        }
    }
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let renderer = Render::new(window.clone());

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent)  {
        let renderer = self.renderer.as_mut().unwrap();
        let _window = self.window.as_mut().unwrap();

        let dt = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();
        self.time += dt;

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }

            WindowEvent::RedrawRequested => {
               renderer.render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[repr(C)]
#[derive(Copy,Clone,bytemuck::Pod, bytemuck::Zeroable)]
struct Camera{
    num:f32,
}

