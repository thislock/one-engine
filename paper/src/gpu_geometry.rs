use wgpu::{util::DeviceExt, RenderPass, VertexBufferLayout};

use crate::engine;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub tex_coords: [f32; 2],
}

impl Vertex {
  const ATTRIBS: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

  #[allow(unused)]
  pub fn desc() -> VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &Self::ATTRIBS,
    }
  }
}

pub struct MeshBuilder {
  vertex_buffer: Option<wgpu::Buffer>,
  index_buffer: Option<wgpu::Buffer>,
  texture_id: Option<String>,
}

impl MeshBuilder {
  pub fn new() -> Self {
    Self {
      vertex_buffer: None,
      index_buffer: None,
      texture_id: None,
    }
  }

  pub fn build(&self, device: wgpu::Device) -> Mesh {

    if let Some(vert_buffer) = self.vertex_buffer {

    }

    todo!()

  }
}

pub struct Mesh {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_indicies: u32,
}

impl Mesh {
  pub fn new(vertices: Vec<Vertex>, indicies: Vec<u32>, device: &wgpu::Device) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: Self::format_vertices(&vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: Self::format_indicies(&indicies),
      usage: wgpu::BufferUsages::INDEX,
    });
    
    Self {
      vertex_buffer,
      index_buffer,
      num_indicies: indicies.len() as u32,
    }
  }

  pub fn format_vertices(vert: &Vec<Vertex>) -> &[u8] {
    bytemuck::cast_slice(vert)
  }

  pub fn format_indicies(indicies: &Vec<u32>) -> &[u8] {
    bytemuck::cast_slice(indicies)
  }

  const TEXTURE_BINDGROUP: u32 = 0;
  const CAMERA_TRANSFORM_BINDGROUP: u32 = 1;

  pub fn render_mesh(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    render_pass.set_bind_group(Self::TEXTURE_BINDGROUP, engine.texture_bundle.get_diffuse_bind_group("yees"), &[]);
    render_pass.set_bind_group(Self::CAMERA_TRANSFORM_BINDGROUP, &engine.camera.camera_bind_group, &[]);

    render_pass.draw_indexed(0..self.num_indicies, 0, 0..1);
  }
}
