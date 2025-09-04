use crate::gpu::shaders::ShaderPipeline;

pub struct Light {
  shader: ShaderPipeline,
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

impl Light {
  
}