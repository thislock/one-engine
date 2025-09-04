use std::sync::Arc;

use wgpu::{util::DeviceExt, RenderPass};
use crate::gpu::geometry::{Vertex, vertex_list_as_bytes};
use crate::gpu::object::SharedLocation;
use crate::gpu::render::{self, RenderTask};
use crate::{
  engine,
  gpu::{
    device_drivers::Drivers,
    //instances::{self, Instance},
    material::Material,
  },
};

pub struct MeshBuilder {
  vertices: Vec<Vertex>,
  indicies: Vec<u32>,
}

impl MeshBuilder {
  pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
    Self {
      vertices,
      indicies: indices,
    }
  }

  pub fn build(
    self,
    drivers: &Drivers,
    material: Material,
    object_location: SharedLocation,
  ) -> anyhow::Result<Mesh> {
    let mesh = Mesh::new(self, &drivers.device, material, object_location);
    Ok(mesh)
  }
}

#[derive(Clone)]
pub struct Mesh {
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  num_indicies: u32,
  material: Material,
  shared_location: SharedLocation,
}

impl Mesh {
  pub fn change_material(&mut self, new_material: Material) {
    self.material = new_material;
  }

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

  fn new(
    mesh_builder: MeshBuilder,
    device: &wgpu::Device,
    material: Material,
    object_location: SharedLocation,
  ) -> Self {
    let vertex_buffer = Self::create_vertex_buffer(&mesh_builder, device);
    let index_buffer = Self::create_index_buffer(&mesh_builder, device);

    // let instance_buffer = Self::add_optional_instances(&mesh_builder, device);

    Self {
      vertex_buffer,
      index_buffer,
      num_indicies: mesh_builder.indicies.len() as u32,
      material,
      shared_location: object_location,
    }
  }

  const TEXTURE_BINDGROUP: u32 = 0;
  const CAMERA_TRANSFORM_BINDGROUP: u32 = 1;
  const TIME_BINDGROUP: u32 = 2;
  const LOCATION_BINDGROUP: u32 = 3;
  fn set_universal_bind_groups(render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    // set the camera transform
    render_pass.set_bind_group(
      Self::CAMERA_TRANSFORM_BINDGROUP,
      &engine.camera.camera_bind_group,
      &[],
    );

    // current time
    render_pass.set_bind_group(Self::TIME_BINDGROUP, &engine.gpu_time.bindgroup, &[]);
  }

  fn set_local_bind_groups(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    // dynamic position
    RenderTask::write_to_buffer(
      engine,
      engine.render_task.get_location_buffer(),
      &[self.shared_location.get_location_ref().to_uniform()],
    );
    render_pass.set_bind_group(
      Self::LOCATION_BINDGROUP,
      engine.render_task.get_location_bindgroup(),
      &[],
    );

    // set the diffuse texture
    render_pass.set_bind_group(Self::TEXTURE_BINDGROUP, &self.material.diffuse_texture, &[]);
  }

  fn set_bind_groups(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    Self::set_universal_bind_groups(render_pass, engine);
    self.set_local_bind_groups(render_pass, engine);
  }

  fn set_geometry_buffers(&self, render_pass: &mut RenderPass<'_>) {
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
  }

  fn submit_for_rendering(&self, render_pass: &mut RenderPass<'_>) {
    render_pass.draw_indexed(0..self.num_indicies, 0, 0..1);
  }

  pub fn render_meshes(
    meshes: &Vec<Arc<Self>>,
    render_pass: &mut RenderPass<'_>,
    engine: &engine::Engine,
  ) {
    Self::set_universal_bind_groups(render_pass, engine);

    for mesh in meshes {
      mesh.set_local_bind_groups(render_pass, engine);
      mesh.set_geometry_buffers(render_pass);
      mesh.submit_for_rendering(render_pass);
    }
  }

  pub fn render_mesh(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {
    self.set_geometry_buffers(render_pass);
    self.set_bind_groups(render_pass, engine);
    self.submit_for_rendering(render_pass);
  }
}

// leaving this dead instance code here

// let instance_range = 0..1;
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

// i've disabled instances for now, because it hate dealing with them, and i don't
// see myself using this any time soon for the engine, also i hate them. did i mention i hate them?
// #[allow(unused)]
// fn create_instance_buffer(mesh_builder: &MeshBuilder, device: &wgpu::Device) -> wgpu::Buffer {
//   let instance_data: Vec<instances::InstanceRaw> = mesh_builder
//     .instances
//     .as_ref()
//     .unwrap()
//     .iter()
//     .map(|x| x.to_raw())
//     .collect();
//   let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//     label: Some("Instance Buffer"),
//     contents: bytemuck::cast_slice(&instance_data),
//     usage: wgpu::BufferUsages::VERTEX,
//   });
//   instance_buffer
// }
// #[allow(unused)]
// fn add_optional_instances(
//   mesh_builder: &MeshBuilder,
//   device: &wgpu::Device,
// ) -> Option<wgpu::Buffer> {
//   let instance_buffer;
//   if mesh_builder.instances.is_some() {
//     instance_buffer = Some(Self::create_instance_buffer(&mesh_builder, device));
//   } else {
//     instance_buffer = None;
//   }
//   return instance_buffer;
// }
