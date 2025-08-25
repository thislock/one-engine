use crate::gpu::geometry::VertexTrait;

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

impl VertexTrait for Instance {
  fn as_bytes(&self) -> Vec<u8> {
    // TODO: make this actually do something.
    let mut bytes = vec![];
    let floats = [
      self.pos.x,
      self.pos.y,
      self.pos.z,
      self.rot.s,
      self.rot.v.x,
      self.rot.v.y,
      self.rot.v.z,
    ];
    return bytes;
  }

  fn desc() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
      // We need to switch from using a step mode of Vertex to Instance
      // This means that our shaders will only change to use the next
      // instance when the shader starts processing a new instance
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
        // for each vec4. We'll have to reassemble the mat4 in the shader.
        wgpu::VertexAttribute {
          offset: 0,
          // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
          // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
          shader_location: 5,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 6,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 7,
          format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 8,
          format: wgpu::VertexFormat::Float32x4,
        },
      ],
    }
  }
}

// for feeding the stupid gpu it's stupid numbers stupid
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
  model: [[f32; 4]; 4],
}
