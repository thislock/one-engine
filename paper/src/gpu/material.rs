#[derive(Clone)]
pub struct Material {
  pub diffuse_texture: wgpu::BindGroup,
}

impl Material {
  pub fn new_basic(diffuse: wgpu::BindGroup) -> Self {
    Self {
      diffuse_texture: diffuse,
    }
  }
}
