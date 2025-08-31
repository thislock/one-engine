use std::sync::Arc;
use crate::gpu::texture::ImageTexture;

pub struct Material {
  diffuse_texture: Arc<ImageTexture>,
}

impl Material {
  pub fn new(diffuse: Arc<ImageTexture>) -> Self {
    Self {
      diffuse_texture: diffuse,
    }
  } 
}