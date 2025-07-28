pub struct Instance {
  pub pos: cgmath::Vector3<f32>,
  pub rot: cgmath::Quaternion<f32>,
}

impl Instance {
  pub fn to_raw(&self) -> InstanceRaw {
    InstanceRaw {
      model: (cgmath::Matrix4::from_translation(self.pos) * cgmath::Matrix4::from(self.rot)).into(),
    }
  }
}

// for feeding the stupid gpu it's stupid numbers stupid
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
  model: [[f32; 4]; 4],
}
