use std::iter;
use wgpu::RenderPass;

use crate::{
  engine,
  gpu::{
    device_drivers,
    object::Object,
    raw_bindgroups,
    shaders::{ShaderBuilder, ShaderBundle, ShaderPipeline},
    texture,
  },
};
pub struct RenderTask {
  pub objects: Vec<Object>,
  shaders: ShaderBundle,
}

impl RenderTask {
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

  pub fn new() -> anyhow::Result<Self> {
    Ok(Self {
      objects: vec![],
      shaders: ShaderBundle::new(),
    })
  }

  fn render_buffers(&self, mut render_pass: RenderPass<'_>, engine: &engine::Engine) {
    for shader in self.shaders.iter_shaders() {
      render_pass.set_pipeline(&shader.render_pipeline);
      for mesh in &shader.meshes {
        mesh.render_mesh(&mut render_pass, engine);
      }
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
