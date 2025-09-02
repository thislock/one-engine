use std::cell::RefCell;
use std::ffi::OsStr;
use std::io::{BufReader, Cursor};
use std::sync::Arc;

use crate::files::{self, load_obj_str};
use crate::gpu::device_drivers::Drivers;
use crate::gpu::geometry::{ModelVertex, Vertex, VertexTrait};
use crate::gpu::texture::TextureBundle;
use crate::gpu::{material, mesh};
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// aligned to 16 bytes
pub struct LocationUniform {
  location_projection: [[f32; 4]; 4],
}

#[derive(Clone, Copy)]
pub struct Location {
  pub pos: Vec3,
  pub rot: Rot,
}

impl Location {

  pub fn to_uniform(&self) -> LocationUniform {
    use cgmath::Matrix4;

    // convert quaternion into a rotation matrix
    let rot_mat: Matrix4<f32> = Matrix4::from(self.rot);
    
    // make a translation matrix
    let trans_mat = Matrix4::from_translation(self.pos);
    
    // combine them: translate * rotate (scale can go here too if needed)
    let model = trans_mat * rot_mat;

    // convert into [[f32; 4]; 4] for uniforms
    LocationUniform { location_projection: model.into() }
  }

  #[inline]
  pub fn new_world_origin() -> Self {
    Self { pos: Object::DEFAULT_POSITION, rot: Object::DEFAULT_ROTATION }
  }
  
  #[inline]
  pub fn from_pos(pos: Vec3) -> Self {
    Self { pos, rot: Object::DEFAULT_ROTATION }
  }

  #[inline]
  pub fn to_shared(self) -> SharedLocation {
    SharedLocation { last_known: self.clone(), shared_known: Arc::new(RefCell::new(self)) }
  }

}

#[derive(Clone)]
pub struct SharedLocation {
  last_known: Location,
  shared_known: Arc<RefCell<Location>>,
}

impl SharedLocation {

  pub fn inc_x(&mut self) {
    self.shared_known.borrow_mut().pos.x += 100.0;
  }

  #[inline]
  pub fn from_location(location: Location) -> Self {
    location.to_shared()
  }

  pub fn get_location<'a>(&'a self) -> &'a Location {
    // this causes a race condition, but i do not care. it seriously doesn't matter for this sort of thing.
    let loc_possible = unsafe { self.shared_known.try_borrow_unguarded() };
    if let Ok(location) = loc_possible {
      return location;
    } else {
      return &self.last_known;
    }
  }

}

pub struct Object {
  pub meshes: Vec<Arc<mesh::Mesh>>,
  pub shared_location: SharedLocation,
}

impl Object3D for Object {
  fn get_pos(&self) -> &Vec3 {
    return &self.shared_location.get_location().pos;
  }
  fn get_rot(&self) -> &Rot {
    return &self.shared_location.get_location().rot;
  }
}

fn load_string(file_name: &str) -> anyhow::Result<String> {
  let str = load_obj_str(file_name)?;
  return Ok(str);
}

pub struct ObjectBuilder {
  meshes: Vec<mesh::Mesh>,

  diffuse: Option<wgpu::BindGroup>,

  global_location: SharedLocation,
}

impl ObjectBuilder {
  fn when_some<T>(possible: Option<T>, mut logic: impl FnMut(T)) {
    if let Some(real) = possible {
      return logic(real);
    }
  }

  fn arcify_vec<T>(vector: Vec<T>) -> Vec<Arc<T>> {
    vector.into_iter().map(Arc::new).collect()
  }

  pub fn new() -> Self {
    Self {
      meshes: Vec::new(),
      diffuse: None,
      global_location: Location::new_world_origin().to_shared(),
    }
  }

  pub fn add_diffuse_texture(mut self, diffuse: wgpu::BindGroup) -> Self {
    self.diffuse = Some(diffuse);
    self
  }

  pub fn set_location(mut self, location: Location) -> Self {
    self.global_location = location.to_shared();
    self
  }

  pub fn set_shared_location(mut self, location: SharedLocation) -> Self {
    self.global_location = location;
    self
  }

  pub fn load_meshes_from_objfile(
    mut self,
    texture_bundle: &TextureBundle,
    drivers: &Drivers,
    file_name: &str,
  ) -> anyhow::Result<Self> {
    let shared_location = self.global_location.clone();
    let object = Object::from_obj_file(texture_bundle, drivers, file_name, &shared_location)?;
    self.meshes.extend(object);
    Ok(self)
  }

  pub fn build(mut self) -> Object {
    Self::when_some(self.diffuse, |diffuse_bind| {
      let material = material::Material::new_basic(diffuse_bind);
      self.meshes.iter_mut().for_each(|mesh| {
        mesh.change_material(material.clone());
      });
    });

    Object {
      meshes: Self::arcify_vec(self.meshes),
      shared_location: Location::new_world_origin().to_shared()
    }
  }
}

impl Object {

  pub fn extract_meshes(self) -> Vec<mesh::Mesh> {
    let mut extracted = Vec::with_capacity(self.meshes.capacity());
    self
      .meshes
      .into_iter()
      .for_each(|mesh| extracted.push((*mesh).clone()));
    return extracted;
  }

  fn from_obj_file(
    texture_bundle: &TextureBundle,
    drivers: &Drivers,
    filename: &str,
    shared_location: &SharedLocation,
  ) -> anyhow::Result<Vec<mesh::Mesh>> {
    use std::io::{BufReader, Cursor};

    let obj_text = files::load_obj_str(filename)?;
    let obj_cursor = Cursor::new(obj_text);
    let obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = Self::get_file_info(obj_reader)?;
    let obj_materials = obj_materials?;

    Self::load_materials(obj_materials)?;

    let meshes = Self::load_meshes(texture_bundle, drivers, models, shared_location);

    Ok(meshes)
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

  /// basically does nothing right now
  #[allow(unused)]
  fn load_materials(obj_materials: Vec<tobj::Material>) -> anyhow::Result<()> {
    for m in obj_materials {
      let diffuse_texture_name = m.diffuse_texture;
    }

    Ok(())
  }

  fn load_meshes(
    texture_bundle: &TextureBundle,
    drivers: &Drivers,
    models: Vec<tobj::Model>,
    shared_location: &SharedLocation
  ) -> Vec<mesh::Mesh> {
    let meshes = models
      .into_iter()
      .map(|m| {
        let vertices = Self::obj_to_vertexes(&m);

        let fallback_texture_binds = texture_bundle
          .get_fallback_texture()
          .diffuse_bind_group
          .clone();
        let material = material::Material::new_basic(fallback_texture_binds);

        let mesh = mesh::MeshBuilder::new(vertices, m.mesh.indices)
          .build(drivers, material, shared_location.clone())
          .unwrap();

        mesh
      })
      .collect::<Vec<_>>();
    meshes
  }

  fn get_file_info(
    mut obj_reader: std::io::BufReader<std::io::Cursor<String>>,
  ) -> Result<
    (
      Vec<tobj::Model>,
      Result<Vec<tobj::Material>, tobj::LoadError>,
    ),
    anyhow::Error,
  > {
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
    Ok((models, obj_materials))
  }
}
