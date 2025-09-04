use wgpu::BindGroupLayout;

use crate::gpu::geometry::GetBufferLayout;

pub struct MemoryLayouts {
  binds: Vec<wgpu::BindGroupLayout>,
}

impl MemoryLayouts {
  pub fn new() -> Self {
    Self { binds: vec![] }
  }

  pub fn add_bind(&mut self, bindable: &dyn GetBufferLayout) {
    self.binds.push(bindable.get_bind_layout());
  }

  pub fn add_bind_raw(&mut self, bindable: &wgpu::BindGroupLayout) {
    self.binds.push(bindable.clone());
  }

  pub fn collect_slice<'a>(&self) -> Vec<&BindGroupLayout> {
    let binding = &self.binds;
    let layout: Vec<&wgpu::BindGroupLayout> = binding.iter().collect();
    return layout;
  }
}
