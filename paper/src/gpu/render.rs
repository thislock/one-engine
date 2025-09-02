use std::iter;
use wgpu::{util::DeviceExt, RenderPass};

use crate::{
  engine,
  gpu::{
    device_drivers,
    geometry::GetBufferLayout,
    mesh,
    object::{self, Object},
    raw_bindgroups,
    shaders::{ShaderBuilder, ShaderBundle, ShaderPipeline},
    texture,
  },
};
pub struct RenderTask {
  pub objects: Vec<Object>,
  shaders: ShaderBundle,

  object_location_bindgroup: wgpu::BindGroup,
  object_location_layout: wgpu::BindGroupLayout,
  object_location_buffer: wgpu::Buffer,
}

impl GetBufferLayout for RenderTask {
  fn get_bind_layout(&self) -> wgpu::BindGroupLayout {
    return self.object_location_layout.clone();
  }
}

impl RenderTask {
  #[inline]
  pub fn get_location_buffer<'a>(&'a self) -> &'a wgpu::Buffer {
    return &self.object_location_buffer;
  }

  #[inline]
  pub fn get_location_bindgroup<'a>(&'a self) -> &'a wgpu::BindGroup {
    return &self.object_location_bindgroup;
  }

  pub fn render(&self, engine: &engine::Engine) -> std::result::Result<(), wgpu::SurfaceError> {
    let output = engine.drivers.surface.get_current_texture()?;
    let mut encoder = self.init_encoder(&engine.drivers);
    let render_pass = self.init_render_pass(&output, &mut encoder, &engine.texture_bundle);

    // tell the gpu what buffers to render
    self.render_buffers(render_pass, &engine);

    engine
      .render_task
      .finish_rendering(output, encoder, &engine.drivers);

    Ok(())
  }

  pub async fn add_object(
    &mut self,
    object: Object,
    drivers: &device_drivers::Drivers,
    bind_groups: &raw_bindgroups::BindGroups,
  ) -> anyhow::Result<()> {
    let shader_builder = ShaderBuilder::from_file("sample.wgsl".to_owned());
    let mut shader = ShaderPipeline::from_shader(bind_groups, drivers, shader_builder).await?;

    for mesh in object.meshes.clone() {
      shader.meshes.push(mesh);
    }

    self.objects.push(object);
    self.shaders.add_shader(shader)?;

    Ok(())
  }

  pub fn write_to_buffer<T>(engine: &engine::Engine, buffer: &wgpu::Buffer, data: &[T])
  where
    T: bytemuck::NoUninit,
  {
    engine
      .drivers
      .queue
      .write_buffer(&buffer, 0, bytemuck::cast_slice(data));
  }

  fn init_bind_group_layout(drivers: &device_drivers::Drivers) -> wgpu::BindGroupLayout {
    drivers
      .device
      .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: Some("object_position_bind_group_layout"),
      })
  }

  fn init_object_bindings(
    drivers: &device_drivers::Drivers,
  ) -> (wgpu::BindGroup, wgpu::BindGroupLayout, wgpu::Buffer) {
    let nil_location = object::Location::new_world_origin();

    let bind_layout = Self::init_bind_group_layout(drivers);

    let loc_buffer = drivers
      .device
      .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Location Buffer"),
        contents: bytemuck::cast_slice(&[nil_location.to_uniform()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    let layout = drivers
      .device
      .create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: loc_buffer.as_entire_binding(),
        }],
        label: Some("object_location_bind_group"),
      });

    (layout, bind_layout, loc_buffer)
  }

  pub fn new(drivers: &device_drivers::Drivers) -> Self {
    let (object_location_bindgroup, object_location_layout, object_location_buffer) =
      Self::init_object_bindings(drivers);
    Self {
      objects: vec![],
      shaders: ShaderBundle::new(),

      object_location_bindgroup,
      object_location_buffer,
      object_location_layout,
    }
  }

  fn render_buffers(&self, mut render_pass: RenderPass<'_>, engine: &engine::Engine) {
    // loop through each shader, and render it's corresponding objects.
    for shader in self.shaders.iter_shaders() {
      render_pass.set_pipeline(&shader.render_pipeline);
      mesh::Mesh::render_meshes(&shader.meshes, &mut render_pass, engine);
    }
  }

  fn finish_rendering(
    &self,
    output: wgpu::SurfaceTexture,
    encoder: wgpu::CommandEncoder,
    drivers: &device_drivers::Drivers,
  ) {
    let command_buffer = encoder.finish();
    drivers.queue.submit(iter::once(command_buffer));
    output.present();
  }

  fn init_encoder(&self, drivers: &device_drivers::Drivers) -> wgpu::CommandEncoder {
    drivers
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      })
  }

  fn init_render_pass<'a>(
    &self,
    output: &wgpu::SurfaceTexture,
    encoder: &'a mut wgpu::CommandEncoder,
    texture_bundle: &texture::TextureBundle,
  ) -> RenderPass<'a> {
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &texture_bundle.depth_buffer.view,
        depth_ops: Some(wgpu::Operations {
          load: wgpu::LoadOp::Clear(1.0),
          store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
      }),
      occlusion_query_set: None,
      timestamp_writes: None,
    });
    return render_pass;
  }
}
