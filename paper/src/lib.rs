use std::sync::Arc;
extern crate sdl3;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;

// binds everything together
#[path = "1engine.rs"]
mod engine;

#[path = "tasks/lib.rs"]
mod task_lib;

#[path = "error.rs"]
mod paper_error;

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

  engine.tickrate.tick();
}

// match event {
//       WindowEvent::CloseRequested => {
//         event_loop.exit();
//       }

//       WindowEvent::Resized(size) => engine.resize(size.width, size.height),
//       WindowEvent::RedrawRequested => on_redraw(engine),
//       #[allow(unused_variables)]
//       WindowEvent::CursorMoved { position, .. } => {}

//       WindowEvent::KeyboardInput {
//         event:
//           KeyEvent {
//             physical_key: PhysicalKey::Code(_code),
//             state: _key_state,
//             ..
//           },
//         ..
//       } => engine.handle_key(&event),
//       _ => {}
//     }

pub async fn run() -> anyhow::Result<()> {
  env_logger::init();

  let sdl_context = sdl3::init()?;
  let video_subsystem = sdl_context.video()?;
  let window = video_subsystem
    .window("Raw Window Handle Example", 800, 600)
    .position_centered()
    .resizable()
    .metal_view()
    .build()?;

  let window = Arc::new(window);

  let mut engine = engine::Engine::new(window.clone()).await;

  let mut event_pump = sdl_context.event_pump()?;
  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Window {
          window_id,
          win_event:
            WindowEvent::PixelSizeChanged(width, height) | WindowEvent::Resized(width, height),
          ..
        } if window_id == window.id() => {
          engine.drivers.surface_config.width = width as u32;
          engine.drivers.surface_config.height = height as u32;
          engine
            .drivers
            .surface
            .configure(&engine.drivers.device, &engine.drivers.surface_config);
        }
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => {
          break 'running;
        }
        e => {
          dbg!(e);
        }
      }
    }

    on_redraw(&mut engine);
  }

  Ok(())
}
