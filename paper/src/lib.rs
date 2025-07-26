use winit::event_loop::EventLoop;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::Window,
};


// binds everything together
#[path = "1engine.rs"]
mod engine;

#[path="tasks/lib.rs"]
mod task_lib;

mod device_drivers;

mod gpu_texture;
mod gpu_geometry;
mod camera;
mod camera_uniform_buffer;

mod tasks;
mod render;

mod gpu_bindgroups;
mod gpu_pipeline;

mod tickrate;
mod z_missing_texture;

struct App {
    engine: Option<engine::Engine>,
}

impl App {
  fn new() -> Self {
    Self {
      engine: None,
    }
  }
}


impl ApplicationHandler<engine::Engine> for App {
    
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes = Window::default_attributes();
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
    
    self.engine = Some(pollster::block_on(engine::Engine::new(window)));
  }

  fn user_event(&mut self, _event_loop: &ActiveEventLoop, engine: engine::Engine) {
    self.engine = Some(engine);
  }

  fn window_event(
      &mut self,
      event_loop: &ActiveEventLoop,
      _window_id: winit::window::WindowId,
      event: WindowEvent,
  ) {
    let engine = match &mut self.engine {
        Some(canvas) => canvas,
        None => return,
    };
    match event {
        
      WindowEvent::CloseRequested => {
          event_loop.exit();
      },
      
      WindowEvent::Resized(size) => engine.resize(size.width, size.height),
      WindowEvent::RedrawRequested => on_redraw(engine),
      #[allow(unused_variables)]
      WindowEvent::CursorMoved { position, .. } => {
      }
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(_code),
            state: _key_state,
            ..
          },
        ..
      } => engine.handle_key(&event),
      _ => {}
    }
  }
  
  fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
    let _ = (event_loop, cause);
  }
  
  fn device_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    device_id: DeviceId,
    event: DeviceEvent,
  ) {
    let _ = (event_loop, device_id, event);
  }
  
  fn exiting(&mut self, event_loop: &ActiveEventLoop) {
    let _ = event_loop;
  }
  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    let _ = event_loop;
  }
  
  // at the time of writing this, these are only for mobile devices (ios, android)
  
  fn suspended(&mut self, event_loop: &ActiveEventLoop) {
    let _ = event_loop;
  }
  
  fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
    let _ = event_loop;
  }

}

fn on_redraw(engine: &mut engine::Engine) {
    engine.update();

    match engine.render_task.render(&engine) {

        Ok(_) => {}
        // Reconfigure the surface if it's lost or outdated
        
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            let size = engine.get_window().inner_size();
            engine.resize(size.width, size.height);
        }
        
        Err(e) => {
            log::error!("Unable to render {}", e);
        }
    
    }

    engine.tickrate.tick();
}

pub fn run() -> anyhow::Result<()> {
    
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    // tells the event loop to run in the background
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}