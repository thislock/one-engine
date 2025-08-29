use std::collections::HashMap;

use crate::gpu::texture;

pub struct Material {
  name: String,
  diffuse_texture: texture::ImageTexture,
  bind_group: wgpu::BindGroup,
}

pub struct MaterialDictionary {
  materials: HashMap<String, Material>,
}

impl MaterialDictionary {
  pub fn new() -> Self {
    Self { 
      materials: HashMap::new() 
    }
  }

  pub fn add(&mut self) {
    //self.materials.insert(k, v)
  }
}