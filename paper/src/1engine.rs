// gets every process and organises it's types to be ran

use std::{sync::Arc, time::Duration};

use crate::{
  camera, device_drivers, gpu_bindgroups, gpu_pipeline, gpu_texture, render, task_lib,
  tasks::{self, LoopGroup},
  tickrate,
};

pub struct Engine {
  pub data_pipeline: gpu_pipeline::PipelineData,

  #[allow(unused)]
  pub data_bindgroups: gpu_bindgroups::BindGroups,
  pub texture_bundle: gpu_texture::TextureBundle,

  pub camera: camera::GpuCamera,
  pub drivers: device_drivers::Drivers,

  pub tickrate: tickrate::Tickrate,
  pub render_task: render::RenderTask,

  pub task_service: tasks::TaskService,

  pub loop_group: LoopGroup,
}

impl Engine {
  async fn new_closed(window: Arc<winit::window::Window>) -> Self {
    let mut data_bindgroups = gpu_bindgroups::BindGroups::new();
    let drivers = device_drivers::Drivers::new(window.clone()).await;
    let render_task = render::RenderTask::new(&drivers).expect("failed to load rendertask");

    let loop_group = LoopGroup::new(Duration::from_secs_f64(1.0 / 60.0));

    let cam = camera::GpuCamera::new(&drivers.device, window.inner_size(), loop_group.clone());
    let texture_bundle = gpu_texture::TextureBundle::new(&drivers.device, &drivers.queue)
      .expect("failed to load texture buffer");

    data_bindgroups.add_bind(texture_bundle.get_texture_bind_group().clone());
    data_bindgroups.add_bind(cam.camera_bind_group_layout.clone());

    let data_pipeline = gpu_pipeline::PipelineData::new(&data_bindgroups, &drivers)
      .await
      .unwrap();
    let task_service = tasks::TaskService::new(window.clone());
    let tickrate = tickrate::Tickrate::new();

    Self {
      render_task,
      texture_bundle,
      data_bindgroups,
      data_pipeline,
      camera: cam,
      task_service,
      tickrate,
      drivers,

      loop_group,
    }
  }

  pub async fn new(window: Arc<winit::window::Window>) -> Self {
    let mut engine = Self::new_closed(window).await;

    task_lib::init_tasks(&mut engine);

    return engine;
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    if width > 0 && height > 0 {
      self.drivers.surface_config.width = width;
      self.drivers.surface_config.height = height;
      self
        .drivers
        .surface
        .configure(&self.drivers.device, &self.drivers.surface_config);
    }
  }

  pub fn get_window(&self) -> Arc<winit::window::Window> {
    self.drivers.window.clone()
  }

  // ************************************** //
  // ************     TASKS     *********** //
  // ************************************** //

  pub fn update(&mut self) {
    self.camera.update_camera(self.tickrate.get_delta());
    self
      .camera
      .camera_uniform
      .update_view_proj(&self.camera.camera);
    self.drivers.queue.write_buffer(
      &self.camera.camera_buffer,
      0,
      bytemuck::cast_slice(&[self.camera.camera_uniform]),
    );
  }

  pub fn handle_key(&mut self, event: &winit::event::WindowEvent) {
    self.camera.camera_controller.process_events(&event);
  }
}
