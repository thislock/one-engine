use std::sync::Arc;
extern crate sdl3;
use cgmath::Rotation3;
use sdl3::{
  event::{Event, WindowEvent},
};

use crate::{
  gpu::{object::{Location, ObjectBuilder, SharedLocation}, shaders::ShaderBuilder},
  maths::Vec3,
  window::{sdl_handle::SdlHandle, tickrate, user_input::MovementHandler},
};

pub mod files;
pub mod maths;

#[path = "1engine.rs"]
pub mod engine;

pub mod gpu;
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

async fn init_objects(e: &mut engine::Engine, shared: &SharedLocation) -> anyhow::Result<()> {
  e.texture_bundle
    .add_texture_from_file(&e.drivers, "test_bake.png", "test")?;

  let diffuse = e.texture_bundle.get_texture_bind("test");

  let object = ObjectBuilder::new()
    .set_shared_location(shared.clone())
    .load_meshes_from_objfile(&e.texture_bundle, &e.drivers, "test_bake_table.obj")?
    .add_diffuse_texture(diffuse.clone())
    .build();

  e.render_task
    .add_object(object, &e.drivers, &e.data_bindgroups)
    .await?;

  Ok(())
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

    let mut shared = Location::from_pos(Vec3::new(0.0, 0.0, 0.0)).to_shared();

    let loc = Location::new_world_origin();
    let shader = ShaderBuilder::from_file("light.wgsl".to_owned());
    self.engine.render_task.add_light(&self.engine.drivers, loc, [1.0, 0.0, 0.0], shader)?;

    init_objects(&mut self.engine, &shared).await?;

    while self.engine.is_running() {
      benchmark.start_measure();

      shared.modify_location(|loc| {
        let rot_factor: cgmath::Quaternion<f32> =
          cgmath::Quaternion::from_angle_z(cgmath::Deg(1.0));
        loc.rot = loc.rot * rot_factor;
      });

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
