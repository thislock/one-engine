use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::gpu::{
  self, device_drivers::Drivers, mesh, object::{self, SharedLocation}, shaders::{RenderingBundle, ShaderPipeline}
};

pub struct Light {
  shader: ShaderPipeline,
  location: SharedLocation,
  light_buffer: wgpu::Buffer,
  light_binding_layout: wgpu::BindGroupLayout,
  light_binding: wgpu::BindGroup,
}

// lib.rs
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
  position: [f32; 3],
  _stupidfuckingpaddingbecausewgpuisstupidandneeds16bytealignmenttowork: u32,
  color: [f32; 3],
  _thispaddingisbasedthoughbecauseitlikesteamfortress2: u32,
}

impl LightUniform {
  pub fn from_location(loc: &object::Location, color: [f32; 3]) -> Self {
    Self {
      _stupidfuckingpaddingbecausewgpuisstupidandneeds16bytealignmenttowork: 0,
      _thispaddingisbasedthoughbecauseitlikesteamfortress2: 0,
      position: loc.pos.into(),
      color,
    }
  }
}

impl Light {
  fn init_buffer(
    drivers: &Drivers,
    light_location: &object::Location,
    color: [f32; 3],
  ) -> wgpu::Buffer {
    let light_uniform = LightUniform::from_location(light_location, color);

    drivers
      .device
      .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light VB"),
        contents: bytemuck::cast_slice(&[light_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      })
  }

  fn init_bind_group_layout(drivers: &Drivers) -> wgpu::BindGroupLayout {
    drivers
      .device
      .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
        label: None,
      })
  }

  fn init_bind_group(
    drivers: &Drivers,
    buffer: &wgpu::Buffer,
    layout: &wgpu::BindGroupLayout,
  ) -> wgpu::BindGroup {
    drivers
      .device
      .create_bind_group(&wgpu::BindGroupDescriptor {
        layout: layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding(),
        }],
        label: None,
      })
  }

  fn init_gpu_handles(
    drivers: &Drivers,
    light_location: &object::Location,
    light_color: &[f32; 3],
  ) -> (wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout) {
    let buffer = Self::init_buffer(drivers, light_location, light_color.clone());
    let bind_layout = Self::init_bind_group_layout(drivers);
    let bind_group = Self::init_bind_group(drivers, &buffer, &bind_layout);
    return (buffer, bind_group, bind_layout);
  }

  fn init_shader_pipeline(
    drivers: &Drivers,
    shaders: &RenderingBundle,
    shader_module: &wgpu::ShaderModule,
    bind_layout: &wgpu::BindGroupLayout,
  ) -> ShaderPipeline {
    let meshes = shaders.get_meshes();

    let mut bindgroup_data = gpu::gpu_pointers::MemoryLayouts::new();
    bindgroup_data.add_bind_raw(bind_layout);

    let render_pipeline = ShaderPipeline::init_render_pipeline(
      &drivers.device,
      shader_module,
      &drivers.surface_config,
      &bindgroup_data,
    );

    ShaderPipeline {
      render_pipeline,
      meshes,
    }
  }

  pub fn add_mesh(&mut self, mesh: Arc<mesh::Mesh>) {
    self.shader.meshes.push(mesh);
  }

  pub fn new_colored(
    drivers: &Drivers, 
    scene: &RenderingBundle, 
    shader: wgpu::ShaderModule,
    location: object::Location, 
    color: [f32; 3],
  ) -> Self {
    let (buffer, bindgroup, grouplayout) = Self::init_gpu_handles(drivers, &location, &color);
    let shader_pipeline = Self::init_shader_pipeline(drivers, scene, &shader, &grouplayout);
    Self {
      shader: shader_pipeline,
      location: location.to_shared(),
      light_buffer: buffer,
      light_binding_layout: grouplayout,
      light_binding: bindgroup,
    }
  }

}
