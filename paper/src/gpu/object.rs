use std::ffi::OsStr;
use std::sync::Arc;

use crate::files::{self, load_obj_str};
use crate::gpu;
use crate::gpu::device_drivers::Drivers;
use crate::gpu::geometry::{Mesh, MeshBuilder, ModelVertex, Vertex, VertexTrait};
use crate::gpu::texture::TextureBundle;
use crate::maths::Vec3;

pub type Rot = cgmath::Quaternion<f32>;

pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const WORLD_RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);

trait Object3D {
  const OBJECT_UP: Vec3 = WORLD_UP;
  const OBJECT_RIGHT: Vec3 = WORLD_RIGHT;

  const DEFAULT_ROTATION: Rot = Rot::new(0.0, 0.0, 0.0, 0.0);
  const DEFAULT_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

  fn get_pos(&self) -> &Vec3;
  fn get_rot(&self) -> &Rot;
}

pub struct Object {
  pub meshes: Vec<Arc<Mesh>>,

  pub global_pos: Vec3,
  pub global_rot: Rot,
}

impl Object3D for Object {
  fn get_pos(&self) -> &Vec3 {
    return &self.global_pos;
  }
  fn get_rot(&self) -> &Rot {
    return &self.global_rot;
  }
}

fn load_string(file_name: &str) -> anyhow::Result<String> {
  let str = load_obj_str(file_name)?;
  return Ok(str);
}

impl Object {
  pub fn from_obj_file(
    texture_bundle: &mut TextureBundle,
    drivers: &Drivers,
    filename: &str,
  ) -> anyhow::Result<Self> {
    use std::io::{BufReader, Cursor};

    let obj_text = files::load_obj_str(filename)?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf(
      &mut obj_reader,
      &tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
      },
      move |p| {
        let filename = OsStr::to_str(p.file_name().unwrap()).unwrap();
        let mat_text = load_string(filename).unwrap();
        tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
      },
    )?;

    Self::load_materials(texture_bundle, drivers, obj_materials)?;

    let meshes = Self::load_meshes(texture_bundle, drivers, models);

    Ok(Self {
      meshes,
      global_pos: Self::DEFAULT_POSITION,
      global_rot: Self::DEFAULT_ROTATION,
    })
  }

  fn obj_to_vertexes(m: &tobj::Model) -> Vec<Box<dyn VertexTrait>> {
    let vertices: Vec<Vertex> = (0..m.mesh.positions.len() / 3)
      .map(|i| {
        if m.mesh.normals.is_empty() {
          Box::new(ModelVertex {
            pos: [
              m.mesh.positions[i * 3],
              m.mesh.positions[i * 3 + 1],
              m.mesh.positions[i * 3 + 2],
            ],
            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
            normal: [0.0, 0.0, 0.0],
          })
        } else {
          Box::new(ModelVertex {
            pos: [
              m.mesh.positions[i * 3],
              m.mesh.positions[i * 3 + 1],
              m.mesh.positions[i * 3 + 2],
            ],
            tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
            normal: [
              m.mesh.normals[i * 3],
              m.mesh.normals[i * 3 + 1],
              m.mesh.normals[i * 3 + 2],
            ],
          })
        }
      })
      .map(|v: Box<ModelVertex>| v as Vertex)
      .collect();
    vertices
  }

  fn load_materials(
    texture_bundle: &mut TextureBundle,
    drivers: &Drivers,
    obj_materials: Result<Vec<tobj::Material>, tobj::LoadError>,
  ) -> Result<(), anyhow::Error> {
    Ok(for m in obj_materials? {
      let diffuse_texture_name = m
        .diffuse_texture
        .expect("FAILED TO GET DIFFUSE TEXTURE NAME");
      println!("{}", diffuse_texture_name);
      let diffuse_texture_bytes = files::load_image_bytes(&diffuse_texture_name)?;
      texture_bundle.add_texture(drivers, &diffuse_texture_bytes, &diffuse_texture_name)?;
    })
  }

  fn load_meshes(
    texture_bundle: &mut TextureBundle,
    drivers: &Drivers,
    models: Vec<tobj::Model>,
  ) -> Vec<Arc<Mesh>> {
    let meshes = models
      .into_iter()
      .map(|m| {
        let vertices = Self::obj_to_vertexes(&m);

        let material = gpu::material::Material::new_basic(
          texture_bundle.get_diffuse_bind_group("").clone()
        );

        let mesh = MeshBuilder::new(vertices, m.mesh.indices)
          .build(drivers, material)
          .unwrap();

        Arc::new(mesh)
      })
      .collect::<Vec<_>>();
    meshes
  }
}
