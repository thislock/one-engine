use std::sync::Arc;
extern crate sdl3;
use sdl3::{
  event::{Event, WindowEvent},
};

use crate::{
  window::{sdl_handle::SdlHandle, tickrate, user_input::MovementHandler},
};

pub mod files;
pub mod maths;

#[path = "1engine.rs"]
pub mod engine;

pub mod gpu;
pub mod tasks;
pub mod window;

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

pub struct EngineRuntime {
  sdl_handle: SdlHandle,
  pub engine: engine::Engine,
}

impl EngineRuntime {
  pub async fn new_engine() -> anyhow::Result<Self> {
    let sdl_handle = SdlHandle::new()?;
    let engine = engine::Engine::new(&sdl_handle, sdl_handle.sdl_window.clone()).await;

    let new_engine = Self { sdl_handle, engine };

    return Ok(new_engine);
  }

  pub async fn run_engine(mut self) -> anyhow::Result<()> {
    let mut movement_buffer = vec![];
    let mut sys_window = self.sdl_handle.sdl_window.clone();

    let mut benchmark = tickrate::TimeMeasurer::new();

    while self.engine.is_running() {
      benchmark.start_measure();

      movement_buffer.clear();

      for event in self.sdl_handle.event_pump.poll_iter() {
        handle_system_events(&event, &mut sys_window, &mut self.engine);
        MovementHandler::poll_movement(&mut self.engine, &mut movement_buffer, &event);
      }
      MovementHandler::apply_movement(&mut self.engine, &mut movement_buffer);

      self.engine.tickrate.tick();
      self.engine.redraw();
      benchmark.stop_measure();
      //println!("{}", benchmark.get_average());
      self.engine.tickrate.sleep_until_next_frame();
    }

    Ok(())
  }
}
