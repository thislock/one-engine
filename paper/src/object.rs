use crate::{device_drivers::Drivers, gpu_texture, maths::*};

pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const WORLD_RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);

struct Rotation {
  front: Vec3,
  pitch: f32,
  yaw: f32,
}

impl Rotation {
  const fn default() -> Self {
    Rotation {
      front: Vec3::new(0.0, 0.0, 0.0),
      pitch: 0.0,
      yaw: 0.0,
    }
  }
}

trait Object3D {
  const OBJECT_UP: Vec3 = WORLD_UP;
  const OBJECT_RIGHT: Vec3 = WORLD_RIGHT;

  const DEFAULT_ROTATION: Rotation = Rotation::default();
  const DEFAULT_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

  fn get_pos() -> Vec3;
  fn get_rot() -> Rotation;
}

pub struct Material {
  pub name: String,
  pub diffuse_texture: gpu_texture::ImageTexture,
  pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
  pub name: String,
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_elements: u32,
  pub material: usize,
}

pub struct Model {
  pub meshes: Vec<Mesh>,
  pub materials: Vec<Material>,
}

pub struct Object {
  models: Vec<Model>,
}

use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

struct ObjectBundle {}

// pub fn load_model(
//   texture_bundle: gpu_texture::TextureBundle,
//   drivers: &Drivers,
//   texture_id: &str,
// ) -> anyhow::Result<Model> {
//   let obj_cursor = Cursor::new(obj_text);
//   let mut obj_reader = BufReader::new(obj_cursor);

//   let (models, obj_materials) = tobj::load_obj_buf(
//     &mut obj_reader,
//     &tobj::LoadOptions {
//       triangulate: true,
//       single_index: true,
//       ..Default::default()
//     },
//     move |p| tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(obj_text))),
//   )?;

//   let mut materials = Vec::new();
//   for m in obj_materials? {
//     let diffuse_texture = load_texture(&m.diffuse_texture, device, queue)?;
//     let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//       layout,
//       entries: &[
//         wgpu::BindGroupEntry {
//           binding: 0,
//           resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
//         },
//         wgpu::BindGroupEntry {
//           binding: 1,
//           resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
//         },
//       ],
//       label: None,
//     });

//     materials.push(model::Material {
//       name: m.name,
//       diffuse_texture,
//       bind_group,
//     })
//   }

//   let meshes = models
//     .into_iter()
//     .map(|m| {
//       let vertices = (0..m.mesh.positions.len() / 3)
//         .map(|i| {
//           if m.mesh.normals.is_empty() {
//             model::ModelVertex {
//               position: [
//                 m.mesh.positions[i * 3],
//                 m.mesh.positions[i * 3 + 1],
//                 m.mesh.positions[i * 3 + 2],
//               ],
//               tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
//               normal: [0.0, 0.0, 0.0],
//             }
//           } else {
//             model::ModelVertex {
//               position: [
//                 m.mesh.positions[i * 3],
//                 m.mesh.positions[i * 3 + 1],
//                 m.mesh.positions[i * 3 + 2],
//               ],
//               tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
//               normal: [
//                 m.mesh.normals[i * 3],
//                 m.mesh.normals[i * 3 + 1],
//                 m.mesh.normals[i * 3 + 2],
//               ],
//             }
//           }
//         })
//         .collect::<Vec<_>>();

//       let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some(&format!("{:?} Vertex Buffer", file_name)),
//         contents: bytemuck::cast_slice(&vertices),
//         usage: wgpu::BufferUsages::VERTEX,
//       });
//       let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some(&format!("{:?} Index Buffer", file_name)),
//         contents: bytemuck::cast_slice(&m.mesh.indices),
//         usage: wgpu::BufferUsages::INDEX,
//       });

//       model::Mesh {
//         name: file_name.to_string(),
//         vertex_buffer,
//         index_buffer,
//         num_elements: m.mesh.indices.len() as u32,
//         material: m.mesh.material_id.unwrap_or(0),
//       }
//     })
//     .collect::<Vec<_>>();

//   Ok(model::Model { meshes, materials })
// }
