use cgmath::{InnerSpace, Vector3};
use wgpu::util::DeviceExt;

use crate::{maths, user_input::InputType};
#[allow(unused)]
use crate::{
  camera, camera_uniform_buffer,
  tasks::{Task, TaskMessenger},
  tickrate,
};

#[allow(unused)]
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
  cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
  cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
  cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
  cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera {
  pub position: cgmath::Point3<f32>,
  pub yaw_radians: f32,
  pub pitch_radians: f32,
  pub fov_degrees: f32,
  pub aspect: f32,
  pub znear: f32,
  pub zfar: f32,
}

impl Camera {
  pub fn new(position: cgmath::Point3<f32>, fov_degrees: f32, aspect_ratio: (u32, u32)) -> Self {
    Self {
      position,
      fov_degrees,
      yaw_radians: 0.0,
      pitch_radians: 0.0,
      aspect: (aspect_ratio.0 as f32) / (aspect_ratio.1 as f32),
      znear: 0.001,
      zfar: 100.0,
    }
  }

  pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    let forward = self.forward_vector();
    let target = self.position + forward;
    let up = cgmath::Vector3::new(0.0, 1.0, 0.0);

    let view = cgmath::Matrix4::look_at_rh(self.position, target, up);
    let proj = cgmath::perspective(
      cgmath::Deg(self.fov_degrees),
      self.aspect,
      self.znear,
      self.zfar,
    );

    proj * view
  }

  pub fn forward_vector(&self) -> Vector3<f32> {
    maths::Vec3::new(
      self.yaw_radians.cos() * self.pitch_radians.cos(),
      self.pitch_radians.sin(),
      self.yaw_radians.sin() * self.pitch_radians.cos(),
    )
    .normalize()
  }

  pub fn update_camera(&mut self, movement: Vec<&InputType>, speed: f32) {
    use cgmath::{vec3, InnerSpace};

    // Clamp pitch to avoid flipping (about ±85°)
    let max_pitch = std::f32::consts::FRAC_PI_2 - 0.1;
    self.pitch_radians = self.pitch_radians.clamp(-max_pitch, max_pitch);

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
          self.position += forward * speed;
        }
        
        InputType::RotateCamera(dir) => {}
      }
    }

    // // Apply movement
    // if is_forward_pressed {
    //   self.position += forward * speed;
    // }
    // if is_backward_pressed {
    //   self.position -= forward * speed;
    // }
    // if is_right_pressed {
    //   self.position += right * speed;
    // }
    // if is_left_pressed {
    //   self.position -= right * speed;
    // }
    // if is_up_pressed {
    //   self.position += up * speed;
    // }
    // if is_down_pressed {
    //   self.position -= up * speed;
    // }
  }
}

pub struct GpuCamera {
  pub camera: camera::Camera,
  pub camera_uniform: camera_uniform_buffer::CameraUniform,
  pub camera_buffer: wgpu::Buffer,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_bind_group_layout: wgpu::BindGroupLayout,
  pub const_speed: f32,
}

impl GpuCamera {
  pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
    let default_camera_position = cgmath::Point3::new(0.0, 3.0, 1.0);
    let camera = Camera::new(default_camera_position, 45.0, size);

    let mut camera_uniform = camera_uniform_buffer::CameraUniform::new();
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
      const_speed: 2.2,
    }
  }

  pub fn update_camera(&mut self, movement: Vec<&InputType>, delta: f32) {
    self
      .camera
      .update_camera(movement, self.const_speed * delta);
  }
}
