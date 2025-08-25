use std::iter;
use wgpu::RenderPass;

use crate::{
  engine, files,
  gpu::{device_drivers, object::Object, texture},
};
pub struct RenderTask {
  pub objects: Vec<Object>,
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

  pub fn new(
    drivers: &device_drivers::Drivers,
    texture_bundle: &mut texture::TextureBundle,
  ) -> anyhow::Result<Self> {
    {
      let asset_bytes = files::load_image_bytes("yees.png")?;
      texture_bundle.add_texture(drivers, &asset_bytes, "yees")?;
    }

    let obj = Object::from_obj_file(texture_bundle, drivers, "test.obj")?;

    Ok(Self { objects: vec![obj] })
  }

  fn render_buffers(&self, mut render_pass: RenderPass<'_>, engine: &engine::Engine) {
    // da boss (no touchy)
    render_pass.set_pipeline(&engine.data_pipeline.render_pipeline);

    for object in &self.objects {
      for mesh in &object.meshes {
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
