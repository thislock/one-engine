use wgpu::{VertexBufferLayout};

pub trait VertexTrait {
  fn desc() -> VertexBufferLayout<'static>
  where
    Self: Sized;

  fn as_bytes(&self) -> Vec<u8>;
}

pub type Vertex = Box<dyn VertexTrait>;

pub trait GetBufferLayout {
  fn get_bind_layout(&self) -> wgpu::BindGroupLayout;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
  pub pos: [f32; 3],
  pub tex_coords: [f32; 2],
  pub normal: [f32; 3],
}

impl VertexTrait for ModelVertex {
  fn desc() -> VertexBufferLayout<'static> {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
      0 => Float32x3, // pos
      1 => Float32x2, // tex_coords
      2 => Float32x3  // normal
    ];

    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &ATTRIBS,
    }
  }

  fn as_bytes(&self) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    bytes.extend(bytemuck::cast_slice(&self.pos));
    bytes.extend(bytemuck::cast_slice(&self.tex_coords));
    bytes.extend(bytemuck::cast_slice(&self.normal));
    return bytes;
  }
}

pub fn vertex_list_as_bytes(vertex_list: &Vec<Vertex>) -> Vec<u8> {
  if vertex_list.len() == 0 {
    return vec![];
  }
  let mut vertex_bytes = vec![];
  let vertex_byte_count = vertex_list.get(0).unwrap().as_bytes().len();
  vertex_bytes.reserve_exact(vertex_list.len() * vertex_byte_count);
  for vertex in vertex_list {
    vertex_bytes.extend(vertex.as_bytes());
  }
  return vertex_bytes;
}
