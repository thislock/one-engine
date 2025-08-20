// gets every process and organises it's types to be ran

use std::{
  sync::Arc,
  time::{self, Duration},
};

use crate::{
  gpu_layer::{
    camera, device_drivers,
    pipeline_data::PipelineData,
    raw_bindgroups, render,
    sync_data::{self, GpuTime},
    texture,
  },
  task_lib::{
    self, init_tasks,
    tasks::{self, LoopGroup},
  },
  window_layer::{sdl_handle::SdlHandle, tickrate, translate_surface, user_input},
};

pub struct Engine {
  pub data_pipeline: PipelineData,

  #[allow(unused)]
  pub data_bindgroups: raw_bindgroups::BindGroups,
  pub texture_bundle: texture::TextureBundle,

  pub camera: camera::GpuCamera,
  pub drivers: device_drivers::Drivers,

  pub tickrate: tickrate::Tickrate,
  pub render_task: render::RenderTask,

  pub task_service: tasks::TaskService,
  pub loop_group: LoopGroup,

  pub gpu_time: GpuTime,
  pub engine_start_time: time::Instant,

  pub user_input: user_input::MovementHandler,

  is_running: bool,
}

impl Engine {
  pub fn redraw(&mut self) {
    self.update();

    // try and render crap
    match self.render_task.render(&self) {
      Ok(_) => {}

      // reconfigure the surface if it's bad
      Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
        let size = self.get_window().0.size();
        self.resize(size.0, size.1);
      }

      Err(error) => {
        log::error!("Unable to render with message: {}", error);
      }
    }
  }

  async fn new_closed(sdl_handle: &SdlHandle, window: Arc<sdl3::video::Window>) -> Self {
    let mut data_bindgroups = raw_bindgroups::BindGroups::new();
    let drivers = device_drivers::Drivers::new(window.clone()).await;
    let render_task = render::RenderTask::new(&drivers).expect("failed to load rendertask");

    let loop_group = LoopGroup::new(Duration::from_secs_f64(1.0));

    let cam = camera::GpuCamera::new(&drivers.device, window.size());
    let texture_bundle =
      texture::TextureBundle::new(&drivers).expect("failed to load texture bundle");

    let gpu_time = sync_data::create_time_bind_group(&drivers.device);

    data_bindgroups.add_bind(texture_bundle.get_texture_bind_group().clone());
    data_bindgroups.add_bind(cam.camera_bind_group_layout.clone());
    data_bindgroups.add_bind(gpu_time.layout.clone());

    let data_pipeline = PipelineData::new(&data_bindgroups, &drivers).await.unwrap();
    let task_service = tasks::TaskService::new(translate_surface::SyncWindow(window.clone()));
    let tickrate = tickrate::Tickrate::new();

    let user_input = user_input::MovementHandler::new(sdl_handle, window.clone());

    Self {
      render_task,
      texture_bundle,
      data_bindgroups,
      data_pipeline,
      camera: cam,
      task_service,
      tickrate,
      drivers,

      gpu_time,
      engine_start_time: time::Instant::now(),

      user_input,

      loop_group,
      is_running: true,
    }
  }

  pub async fn new(sdl_handle: &SdlHandle, window: Arc<sdl3::video::Window>) -> Self {
    let mut engine = Self::new_closed(sdl_handle, window).await;

    init_tasks(&mut engine);

    return engine;
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      // resize window
      self.camera.set_aspect((width, height));
      self.drivers.surface_config.width = width;
      self.drivers.surface_config.height = height;
      self
        .drivers
        .surface
        .configure(&self.drivers.device, &self.drivers.surface_config);
      // resize textures (make a new one)
      self.texture_bundle.depth_buffer =
        texture::DynamicTexture::create_depth_buffer(&self.drivers);
    }
  }

  pub fn get_window(&self) -> translate_surface::SyncWindow {
    translate_surface::SyncWindow(self.drivers.window.clone())
  }

  // **************************************** //
  // ************     TASKS     ************* //
  // **************************************** //

  pub fn update(&mut self) {
    self
      .camera
      .camera_uniform
      .update_view_proj(&self.camera.camera);
    // write to the camera variable on the gpu
    self.drivers.queue.write_buffer(
      &self.camera.camera_buffer,
      0,
      bytemuck::cast_slice(&[self.camera.camera_uniform]),
    );
    // write the time variable on the gpu
    let secs_since_started = time::Instant::now()
      .duration_since(self.engine_start_time)
      .as_secs_f32();
    self.drivers.queue.write_buffer(
      &self.gpu_time.buffer,
      0,
      bytemuck::cast_slice(&[secs_since_started]),
    );
  }

  // ************************ STARTUP/CLOSING LOGIC ************************** //

  // TODO: for a final engine, don't hard close when the os tries to,
  // make sure to save any game data, or at least ask the player if they're SURE they want to close right now
  pub fn request_close(&mut self) {
    self.is_running = false;
  }

  pub fn is_running(&self) -> bool {
    self.is_running
  }
}
