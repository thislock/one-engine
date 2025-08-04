use crate::{device_drivers, engine, gpu_geometry::{self, Vertex}, gpu_texture, instances};
use std::iter;
use wgpu::RenderPass;
pub struct RenderTask {
  pub mesh: gpu_geometry::Mesh,
}

impl RenderTask {
  pub fn new(drivers: &device_drivers::Drivers) -> anyhow::Result<Self> {
    // texture system
    let mut texture_bundle = gpu_texture::TextureBundle::new(drivers)?;

    texture_bundle.add_texture(drivers, include_bytes!("assets/yees.png"), "yees")?;

    // rendering stuff
    let create_vertex = |pos: [f32; 3], tex_coords: [f32; 2]| -> Vertex {
      Box::new(gpu_geometry::ModelVertex {
        pos,tex_coords,normal: [0.0; 3],
      })
    };
    let vertices: Vec<Vertex> = vec![
      create_vertex([-0.0868241, 0.49240386, 0.0], [0.4131759, 0.99240386]),
      create_vertex([-0.49513406, 0.06958647, 0.0], [0.0048659444, 0.56958647]),
      create_vertex([-0.21918549, -0.44939706, 0.0], [0.28081453, 0.05060294]),
      create_vertex([0.35966998, -0.3473291, 0.0], [0.85967, 0.1526709]),
      create_vertex([0.44147372, 0.2347359, 0.0], [0.9414737, 0.7347359]),
    ];

    let indicies = vec![0, 1, 4, 1, 2, 4, 2, 3, 4];

    use cgmath::prelude::*;

    const NUM_INSTANCES_PER_ROW: u32 = 30;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
      NUM_INSTANCES_PER_ROW as f32 * 0.5,
      0.0,
      NUM_INSTANCES_PER_ROW as f32 * 0.5,
    );

    let instances = (0..NUM_INSTANCES_PER_ROW)
      .flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
          let pos = cgmath::Vector3 {
            x: x as f32,
            y: 0.0,
            z: z as f32,
          } - INSTANCE_DISPLACEMENT;
          let rot = if pos.is_zero() {
            // this is needed so an object at (0, 0, 0) won't get scaled to zero
            // as Quaternions can affect scale if they're not created correctly
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
          } else {
            cgmath::Quaternion::from_axis_angle(pos.normalize(), cgmath::Deg(45.0))
          };

          instances::Instance { pos, rot }
        })
      })
      .collect::<Vec<_>>();

    let mesh = gpu_geometry::MeshBuilder::new(vertices, indicies)
      .add_instances(instances)
      .build(&drivers.device)?;

    Ok(Self { mesh })
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

  fn render_buffers(&self, mut render_pass: RenderPass<'_>, engine: &engine::Engine) {
    // da boss (no touchy)
    render_pass.set_pipeline(&engine.data_pipeline.render_pipeline);

    self.mesh.render_mesh(&mut render_pass, engine);
  }

  fn finish_rendering(
    &self,
    output: wgpu::SurfaceTexture,
    encoder: wgpu::CommandEncoder,
    drivers: &device_drivers::Drivers,
  ) {
    drivers.queue.submit(iter::once(encoder.finish()));
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
    texture_bundle: &gpu_texture::TextureBundle,
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
