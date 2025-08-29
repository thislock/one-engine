use wgpu::{util::DeviceExt, RenderPass, VertexBufferLayout};
use crate::{
  engine,
  gpu::{
    instances::{self, Instance},
  },
};

pub trait VertexTrait {
  fn desc() -> VertexBufferLayout<'static>
  where
    Self: Sized;

  fn as_bytes(&self) -> Vec<u8>;
}

pub type Vertex = Box<dyn VertexTrait>;

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

pub struct MeshBuilder {
  vertices: Vec<Vertex>,
  indicies: Vec<u32>,
  instances: Option<Vec<Instance>>,
  texture_id: Option<String>,
}

impl MeshBuilder {
  pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
    Self {
      vertices,
      indicies: indices,
      texture_id: None,
      instances: None,
    }
  }

  pub fn add_instances(mut self, instances: Vec<Instance>) -> Self {
    self.instances = Some(instances);
    self
  }

  pub fn build(self, device: &wgpu::Device) -> anyhow::Result<Mesh> {
    let texture_id = self.texture_id.clone().unwrap_or(String::from(""));

    let mut mesh = Mesh::new(self, device);

    mesh.material = Material {
      name: "".to_owned(),
      diffuse_texture: texture_id,
    };

    Ok(mesh)
  }
}

pub struct Material {
  pub name: String,
  pub diffuse_texture: String,
}

pub struct Mesh {
  // change this to use materials
  material: Material,

  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  num_indicies: u32,
  // instance_buffer: Option<wgpu::Buffer>,
  // instances: Option<Vec<Instance>>,
}

impl Mesh {
  fn create_vertex_buffer(mesh_builder: &MeshBuilder, device: &wgpu::Device) -> wgpu::Buffer {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: &vertex_list_as_bytes(&mesh_builder.vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });
    vertex_buffer
  }

  fn create_index_buffer(mesh_builder: &MeshBuilder, device: &wgpu::Device) -> wgpu::Buffer {
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(&mesh_builder.indicies),
      usage: wgpu::BufferUsages::INDEX,
    });
    index_buffer
  }

  /// i've disabled instances for now, because it hate dealing with them, and i don't
  /// see myself using this any time soon for the engine, also i hate them. did i mention i hate them?
  #[allow(unused)]
  fn create_instance_buffer(mesh_builder: &MeshBuilder, device: &wgpu::Device) -> wgpu::Buffer {
    let instance_data: Vec<instances::InstanceRaw> = mesh_builder
      .instances
      .as_ref()
      .unwrap()
      .iter()
      .map(|x| x.to_raw())
      .collect();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX,
    });
    instance_buffer
  }
  #[allow(unused)]
  fn add_optional_instances(
    mesh_builder: &MeshBuilder,
    device: &wgpu::Device,
  ) -> Option<wgpu::Buffer> {
    let instance_buffer;
    if mesh_builder.instances.is_some() {
      instance_buffer = Some(Self::create_instance_buffer(&mesh_builder, device));
    } else {
      instance_buffer = None;
    }

    return instance_buffer;
  }

  fn new(mesh_builder: MeshBuilder, device: &wgpu::Device) -> Self {
    let vertex_buffer = Self::create_vertex_buffer(&mesh_builder, device);
    let index_buffer = Self::create_index_buffer(&mesh_builder, device);

    // let instance_buffer = Self::add_optional_instances(&mesh_builder, device);

    Self {
      vertex_buffer,
      index_buffer,
      // instance_buffer,
      // instances: mesh_builder.instances,
      num_indicies: mesh_builder.indicies.len() as u32,
      material: Material {
        name: "".to_owned(),
        diffuse_texture: "".to_owned(),
      },
    }
  }

  const TEXTURE_BINDGROUP: u32 = 0;
  const CAMERA_TRANSFORM_BINDGROUP: u32 = 1;
  const TIME_BINDGROUP: u32 = 2;

  pub fn render_mesh(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    render_pass.set_bind_group(
      Self::TEXTURE_BINDGROUP,
      engine
        .texture_bundle
        .get_diffuse_bind_group(&self.material.diffuse_texture),
      &[],
    );
    render_pass.set_bind_group(
      Self::CAMERA_TRANSFORM_BINDGROUP,
      &engine.camera.camera_bind_group,
      &[],
    );
    render_pass.set_bind_group(Self::TIME_BINDGROUP, &engine.gpu_time.bindgroup, &[]);

    let instance_range = 0..1;
    // if let Some(instance) = &self.instance_buffer {
    //   render_pass.set_vertex_buffer(1, instance.slice(..));
    //   render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //   unsafe { instance_range = 0..self.instances.as_ref().unwrap_unchecked().len() as _; }
    // } else {
    //   // this fix makes it not crash if there isnt instances, but its absolitely amazingly stupid to leave it like this
    //   render_pass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
    //   render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //   instance_range = 0..1;
    // }

    render_pass.draw_indexed(0..self.num_indicies, 0, instance_range);
  }
}
