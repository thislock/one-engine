use wgpu::util::DeviceExt;

pub struct GpuTime {
  pub bindgroup: wgpu::BindGroup,
  pub buffer: wgpu::Buffer,
  pub layout: wgpu::BindGroupLayout,
}

pub fn create_time_bind_group(device: &wgpu::Device) -> GpuTime {
  let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Time Buffer"),
    contents: bytemuck::cast_slice(&[0.0]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
  });

  let time_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    label: Some("time_bind_group_layout"),
  });

  let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &time_bind_group_layout,
    entries: &[wgpu::BindGroupEntry {
      binding: 0,
      resource: time_buffer.as_entire_binding(),
    }],
    label: Some("time_bind_group"),
  });

  GpuTime {
    bindgroup: time_bind_group,
    buffer: time_buffer,
    layout: time_bind_group_layout,
  }
}
