use cgmath::{EuclideanSpace, Vector3};
use wgpu::util::DeviceExt;


use crate::tasks::LoopGroup;
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
  pub eye: cgmath::Point3<f32>,
  pub target: cgmath::Point3<f32>,
  // which axis is up
  pub up: cgmath::Vector3<f32>,
  pub aspect: f32,
  pub fovy: f32,
  pub znear: f32,
  pub zfar: f32,
}

impl Camera {
  pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    // 1.
    let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
    // 2.
    let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    // 3.
    return proj * view;
  }
}

pub struct CameraController {
  speed: f32,
  base_speed: f32,
  is_up_pressed: bool,
  is_down_pressed: bool,
  is_left_pressed: bool,
  is_right_pressed: bool,

  is_forward_pressed: bool,
  is_backward_pressed: bool,
  is_moving_right: bool,
  is_moving_left: bool,
}

impl CameraController {
  fn new(speed: f32) -> Self {
    Self {
      speed,
      base_speed: speed,
      is_down_pressed: false,
      is_up_pressed: false,
      is_left_pressed: false,
      is_right_pressed: false,

      is_forward_pressed: false,
      is_backward_pressed: false,
      is_moving_right: false,
      is_moving_left: false,
    }
  }

  fn get_degrees(rot: f32) -> f32 {
    (rot + 1.0) * 180.0
  }

  #[allow(unused)]
  pub fn print_vec_degrees(rot: &Vector3<f32>) {
    println!(
      "x: {} y: {} z: {}",
      Self::get_degrees(rot.x),
      Self::get_degrees(rot.y),
      Self::get_degrees(rot.z),
    );
  }

  pub fn update_camera(&self, camera: &mut Camera) {
    use cgmath::InnerSpace;
    let forward = camera.eye.to_vec();
    let forward_norm = forward.normalize();
    let forward_mag = forward.magnitude();
    // Prevents glitching when the camera gets too close to the
    // center of the scene.
    if self.is_forward_pressed && forward_mag > self.speed {
      camera.eye -= forward_norm * self.speed;
    }
    if self.is_backward_pressed {
      camera.eye += forward_norm * self.speed;
    }
    let vertical_lock = Vector3 {
      x: 1.0,
      y: 0.0,
      z: 0.0,
    };
    let right = forward_norm.cross(camera.up);
    let up = forward_norm.cross(vertical_lock);
    // Redo radius calc in case the forward/backward is pressed.
    let forward = camera.target - camera.eye;
    let forward_mag = forward.magnitude();
    if self.is_up_pressed {
      camera.eye = camera.target - (forward + up * self.speed).normalize() * forward_mag;
    }
    if self.is_down_pressed {
      camera.eye = camera.target - (forward - up * self.speed).normalize() * forward_mag;
    }
    if self.is_right_pressed {
      // Rescale the distance between the target and the eye so
      // that it doesn't change. The eye, therefore, still
      // lies on the circle made by the target and eye.
      camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
    }
    if self.is_left_pressed {
      camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
    }
  }
}

pub struct GpuCamera {
  pub camera: camera::Camera,
  pub camera_uniform: camera_uniform_buffer::CameraUniform,
  pub camera_buffer: wgpu::Buffer,
  pub camera_bind_group: wgpu::BindGroup,
  pub camera_bind_group_layout: wgpu::BindGroupLayout,
  pub camera_controller: CameraController,

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
    let camera = camera::Camera {
      // position the camera 1 unit up and 2 units back
      // +z is out of the screen
      eye: (0.0, 1.0, 2.0).into(),
      // have it look at the origin
      target: (0.0, 0.0, 0.0).into(),
      // which way is "up"
      up: cgmath::Vector3::unit_y(),
      aspect: size.0 as f32 / size.1 as f32,
      fovy: 45.0,
      znear: 0.1,
      zfar: 100.0,
    };

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
      camera_controller: CameraController::new(3.0),

      loop_group,
    }
  }

  pub fn update_camera(&mut self, delta: f32) {
    self.camera_controller.speed = self.camera_controller.base_speed * delta;
    self.camera_controller.update_camera(&mut self.camera);
  }
}
