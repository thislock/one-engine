use cgmath::{InnerSpace, Vector3};
use wgpu::util::DeviceExt;

use crate::{
  gpu::{geometry::GetBufferLayout, object},
  maths,
  window::user_input::InputType,
};

#[allow(unused)]
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
  cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
  cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
  cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
  cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  // We can't use cgmath with bytemuck directly, so we'll have
  // to convert the Matrix4 into a 4x4 f32 array
  view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
  pub fn new() -> Self {
    use cgmath::SquareMatrix;
    Self {
      view_proj: cgmath::Matrix4::identity().into(),
    }
  }

  pub fn update_view_proj(&mut self, camera: &Camera) {
    self.view_proj = camera.build_view_projection_matrix().into();
  }
}

pub struct Camera {
  pub position: cgmath::Point3<f32>,
  pub yaw_radians: f32,
  pub pitch_radians: f32,
  pub fov_degrees: f32,
  pub aspect: f32,
  pub znear: f32,
  pub zfar: f32,
}

fn get_aspect_from_u32(aspect_ratio: (u32, u32)) -> f32 {
  ((aspect_ratio.0 as f64) / (aspect_ratio.1 as f64)) as f32
}

impl Camera {
  pub fn new(position: cgmath::Point3<f32>, fov_degrees: f32, aspect_ratio: (u32, u32)) -> Self {
    Self {
      position,
      fov_degrees,
      yaw_radians: 0.0,
      pitch_radians: 0.0,
      aspect: get_aspect_from_u32(aspect_ratio),
      znear: 0.001,
      zfar: 100.0,
    }
  }

  fn get_view_projection(&self) -> cgmath::Matrix4<f32> {
    cgmath::perspective(
      cgmath::Deg(self.fov_degrees),
      self.aspect,
      self.znear,
      self.zfar,
    )
  }

  pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    let forward = self.forward_vector();
    let target = self.position + forward;

    let view = cgmath::Matrix4::look_at_rh(self.position, target, object::WORLD_UP);
    let view_projection = self.get_view_projection();

    return view_projection * view;
  }

  pub fn forward_vector(&self) -> Vector3<f32> {
    maths::Vec3::new(
      self.yaw_radians.cos() * self.pitch_radians.cos(),
      self.pitch_radians.sin(),
      self.yaw_radians.sin() * self.pitch_radians.cos(),
    )
    .normalize()
  }

  pub fn update_camera(&mut self, movement: &Vec<InputType>, speed: f32) {
    use cgmath::{vec3, InnerSpace};

    for input in movement {
      match input {
        InputType::MoveCamera(dir) => {
          // Calculate forward vector from yaw/pitch
          let yaw = self.yaw_radians + dir.direction.angle.as_radians() as f32;
          let pitch = self.pitch_radians;
          let forward = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
          )
          .normalize();
          self.position += forward * (speed * dir.direction.magnitude as f32);
        }

        InputType::RotateCamera(dir) => {
          const SENSITIVITY: f64 = 0.1;

          let dx = dir.pitch;
          let dy = dir.yaw;
          self.yaw_radians += (dx * SENSITIVITY) as f32 * speed;
          self.pitch_radians -= (dy * SENSITIVITY) as f32 * speed;

          let max_pitch = std::f32::consts::FRAC_PI_2 - 0.1;
          self.pitch_radians = self.pitch_radians.clamp(-max_pitch, max_pitch);
        }
      }
    }
  }
}

pub struct GpuCamera {
  pub camera: Camera,
  pub camera_uniform: CameraUniform,
  pub camera_buffer: wgpu::Buffer,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl GetBufferLayout for GpuCamera {
  fn get_bind_layout(&self) -> wgpu::BindGroupLayout {
    self.camera_bind_group_layout.clone()
  }
}

impl GpuCamera {
  pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
    let default_camera_position = cgmath::Point3::new(1.0, 0.0, 0.0);
    let camera = Camera::new(default_camera_position, 45.0, size);

    let mut camera_uniform = CameraUniform::new();
    camera_uniform.update_view_proj(&camera);

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Camera Buffer"),
      contents: bytemuck::cast_slice(&[camera_uniform]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
        label: Some("camera_bind_group_layout"),
      });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &camera_bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: camera_buffer.as_entire_binding(),
      }],
      label: Some("camera_bind_group"),
    });

    Self {
      camera,
      camera_uniform,
      camera_buffer,
      camera_bind_group,
      camera_bind_group_layout,
    }
  }

  pub fn update_camera(&mut self, movement: &Vec<InputType>, delta: f32) {
    self.camera.update_camera(movement, delta);
  }

  pub fn set_aspect(&mut self, size: (u32, u32)) {
    self.camera.aspect = get_aspect_from_u32(size);
  }
}
