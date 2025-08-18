use std::{sync::Arc, thread};
extern crate sdl3;
use sdl3::event::{Event, WindowEvent};

use crate::user_input::MovementHandler;

// binds everything together
#[path = "1engine.rs"]
mod engine;

#[path = "tasks/lib.rs"]
mod task_lib;

#[path = "error.rs"]
mod paper_error;

mod user_input;

mod maths;

mod translate_surface;

mod device_drivers;

mod camera;
mod camera_uniform_buffer;
mod gpu_geometry;
mod gpu_sync_data;
mod gpu_texture;
mod instances;

mod object;

mod render;
mod tasks;

mod gpu_bindgroups;
mod gpu_pipeline;

mod tickrate;

#[path = "hardcoded_values/missing_texture.rs"]
mod missing_texture;

fn on_redraw(engine: &mut engine::Engine) {
  engine.update();

  // try and render crap
  match engine.render_task.render(&engine) {
    Ok(_) => {}

    // reconfigure the surface if it's bad
    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
      let size = engine.get_window().0.size();
      engine.resize(size.0, size.1);
    }

    Err(e) => {
      paper_error::log_error("Unable to render", e.into());
    }
  }


}

#[allow(unused)]
pub struct SdlHandle {
  pub sdl_context: sdl3::Sdl,
  pub sdl_window: Arc<sdl3::video::Window>,
  pub event_pump: sdl3::EventPump,
}

impl SdlHandle {
  fn new() -> anyhow::Result<Self> {
    env_logger::init();

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
      .window("one engine demo", 800, 600)
      .position_centered()
      .resizable()
      .metal_view()
      .build()?;

    sdl_context.mouse().set_relative_mouse_mode(&window, true);

    let window = Arc::new(window);

    let event_pump = sdl_context.event_pump()?;

    Ok(Self {
      sdl_context,
      sdl_window: window,
      event_pump,
    })
  }
}

fn handle_system_events(
  event: &sdl3::event::Event,
  window: &mut Arc<sdl3::video::Window>,
  engine: &mut engine::Engine,
) {
  match event {
    Event::Window {
      window_id,
      win_event: WindowEvent::PixelSizeChanged(width, height) | WindowEvent::Resized(width, height),
      ..
    } if *window_id == window.id() => {
      engine.resize(*width as u32, *height as u32);
    }
    Event::Quit { .. } => {
      engine.request_close();
    }
    _ => {}
  }
}

pub async fn run_engine() -> anyhow::Result<()> {
  let mut sdl_handle = SdlHandle::new()?;
  let mut engine = engine::Engine::new(&sdl_handle, sdl_handle.sdl_window.clone()).await;

  let mut movement_buffer = vec![];
  let mut sys_window = sdl_handle.sdl_window.clone();

  while engine.is_running() {
    movement_buffer.clear();

    for event in sdl_handle.event_pump.poll_iter() {
      handle_system_events(&event, &mut sys_window, &mut engine);
      MovementHandler::poll_movement(&mut engine, &mut movement_buffer, &event);
    }
    MovementHandler::apply_movement(&mut engine, &mut movement_buffer);

    // TODO: wait for all keyboard related tasks to finish, THEN render
    on_redraw(&mut engine);
    engine.tickrate.tick();
    engine.tickrate.sleep_until_next_frame();
  }

  Ok(())
}
