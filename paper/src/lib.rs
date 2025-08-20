use std::sync::Arc;
extern crate sdl3;
use sdl3::event::{Event, WindowEvent};

use crate::window_layer::{sdl_handle::SdlHandle, user_input::MovementHandler};

mod files;
mod maths;

#[path = "1engine.rs"]
mod engine;

#[path = "gpu/lib.rs"]
mod gpu_layer;

#[path = "window/lib.rs"]
mod window_layer;

#[path = "tasks/lib.rs"]
mod task_lib;

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

    engine.redraw();
    engine.tickrate.tick();
    engine.tickrate.sleep_until_next_frame();
  }

  Ok(())
}
