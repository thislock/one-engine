use std::sync::Arc;
extern crate sdl3;
use sdl3::{event::{Event, WindowEvent}, keyboard::Keycode};

use crate::{gpu::pipeline_data, window::{sdl_handle::SdlHandle, tickrate, user_input::MovementHandler}};

mod files;
mod maths;

#[path = "1engine.rs"]
mod engine;

mod gpu;
mod tasks;
mod window;

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

  let mut benchmark = tickrate::TimeMeasurer::new();

  while engine.is_running() {
    benchmark.start_measure();

    movement_buffer.clear();

    for event in sdl_handle.event_pump.poll_iter() {
      handle_system_events(&event, &mut sys_window, &mut engine);
      MovementHandler::poll_movement(&mut engine, &mut movement_buffer, &event);
      match event {
        Event::KeyDown { keycode, .. } => {
          if let Some(key) = keycode {
            match key {
              Keycode::_0 => {
                println!("recompiled shaders");
                pipeline_data::PipelineData::recompile_shaders(&mut engine).await?;
              },
              _=>{}
            }
          }
        },
        _ => {},
      }      
    }
    MovementHandler::apply_movement(&mut engine, &mut movement_buffer);

    engine.tickrate.tick();
    engine.redraw();
    benchmark.stop_measure();
    //println!("{}", benchmark.get_average());
    engine.tickrate.sleep_until_next_frame();
  }

  Ok(())
}
