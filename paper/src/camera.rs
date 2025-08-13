use std::time;

use cgmath::{EuclideanSpace, InnerSpace, Vector3};
use wgpu::util::DeviceExt;

use crate::{object, tasks::LoopGroup};
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
    object::Vec3::new(
      self.yaw_radians.cos() * self.pitch_radians.cos(),
      self.pitch_radians.sin(),
      self.yaw_radians.sin() * self.pitch_radians.cos(),
    )
    .normalize()
  }

  pub fn update_camera(
    &mut self,
    delta_yaw: f32,
    delta_pitch: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_right_pressed: bool,
    is_left_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    speed: f32,
  ) {
    use cgmath::{vec3, InnerSpace};

    // Update yaw/pitch from mouse movement
    self.yaw_radians += delta_yaw;
    self.pitch_radians += delta_pitch;

    // Clamp pitch to avoid flipping (about ±85°)
    let max_pitch = std::f32::consts::FRAC_PI_2 - 0.1;
    self.pitch_radians = self.pitch_radians.clamp(-max_pitch, max_pitch);

    // Calculate forward vector from yaw/pitch
    let forward = vec3(
      self.yaw_radians.cos() * self.pitch_radians.cos(),
      self.pitch_radians.sin(),
      self.yaw_radians.sin() * self.pitch_radians.cos(),
    )
    .normalize();

    // Right and up vectors
    let right = forward.cross(vec3(0.0, 1.0, 0.0)).normalize();
    let up = vec3(0.0, 1.0, 0.0);

    // Apply movement
    if is_forward_pressed {
      self.position += forward * speed;
    }
    if is_backward_pressed {
      self.position -= forward * speed;
    }
    if is_right_pressed {
      self.position += right * speed;
    }
    if is_left_pressed {
      self.position -= right * speed;
    }
    if is_up_pressed {
      self.position += up * speed;
    }
    if is_down_pressed {
      self.position -= up * speed;
    }
  }
}

pub struct GpuCamera {
  pub camera: camera::Camera,
  pub camera_uniform: camera_uniform_buffer::CameraUniform,
  pub camera_buffer: wgpu::Buffer,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_bind_group_layout: wgpu::BindGroupLayout,
  pub const_speed: f32,

  loop_group: LoopGroup,
}

impl Task for GpuCamera {
  fn get_type(&self) -> crate::tasks::TaskType {
    crate::tasks::TaskType::Looping(self.loop_group.clone())
  }
  fn run_task(
    &mut self,
    _messages: &mut TaskMessenger,
    // the time since the function was ran last
    delta_time: f32,
  ) -> anyhow::Result<()> {
    self.update_camera(delta_time);
    Ok(())
  }
}

impl GpuCamera {
  pub fn new(device: &wgpu::Device, size: (u32, u32), loop_group: LoopGroup) -> Self {
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

      loop_group,
    }
  }

  pub fn update_camera(&mut self, delta: f32) {
  
    self.camera.update_camera(
      0.01,
      -0.001,
      true,
      false,
      false,
      false,
      false,
      false,
      self.const_speed * delta,
    );
  }
}
