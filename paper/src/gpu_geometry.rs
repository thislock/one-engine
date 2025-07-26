
use wgpu::{util::DeviceExt, VertexBufferLayout};


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

pub struct Mesh {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_indicies: u32, 
}

impl Mesh {

  pub fn new(vertices: Vec<Vertex>, indicies: Vec<u32>, device: &wgpu::Device) -> Self {

    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: Self::format_vertices(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );

    let index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: Self::format_indicies(&indicies),
        usage: wgpu::BufferUsages::INDEX,
      }
    );
    Self {
      vertex_buffer, index_buffer, num_indicies: indicies.len() as u32,
    }
    
  }

  pub fn format_vertices(vert: &Vec<Vertex>) -> &[u8] {
    bytemuck::cast_slice(vert)
  }

  pub fn format_indicies(indicies: &Vec<u32>) -> &[u8] {
    bytemuck::cast_slice(indicies)
  }
}
