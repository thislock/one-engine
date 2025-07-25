use std::iter;
use wgpu::RenderPass;
use crate::{device_drivers, engine, gpu_geometry, gpu_texture};

pub struct RenderTask {
    pub mesh: gpu_geometry::Mesh,
}

impl RenderTask {

    pub fn new(drivers: &device_drivers::Drivers) -> anyhow::Result<Self> {
        // texture system
        let mut texture_bundle = gpu_texture::TextureBundle::new(&drivers.device, &drivers.queue)?;

        texture_bundle.add_texture(&drivers.device, &drivers.queue, include_bytes!("assets/yees.png"), "yees")?;

        // rendering stuff
        let vertices = vec![
            gpu_geometry::Vertex { position: [-0.0868241, 0.49240386, 0.5], tex_coords: [0.4131759, 0.00759614], }, // A
            gpu_geometry::Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
            gpu_geometry::Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
            gpu_geometry::Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
            gpu_geometry::Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
        ];

        let indicies = vec![
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
        ];
        
        let mesh = gpu_geometry::Mesh::new(
            vertices,
            indicies,
            &drivers.device
        );

        Ok(Self {
            mesh,
        })
    }
    
    pub fn render(&self, engine: &engine::Engine) -> std::result::Result<(), wgpu::SurfaceError> {

        let output = engine.drivers.surface.get_current_texture()?;
        let mut encoder = self.init_encoder(&engine.drivers);
        let mut render_pass = self.init_render_pass(&output, &mut encoder);

        // tell the gpu what buffers to render
        self.render_buffers(&mut render_pass, &engine);

        // everything is already passed to the gpu,
        // so we free this to avoid borrow issues
        drop(render_pass);

        engine.render_task.finish_rendering(output, encoder, &engine.drivers);

        Ok(())
    }
    
    fn render_buffers(&self, render_pass: &mut RenderPass<'_>, engine: &engine::Engine) {

        render_pass.set_pipeline(&engine.data_pipeline.render_pipeline);
        
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        
        render_pass.set_bind_group(0, engine.texture_bundle.get_diffuse_bind_group("yees"), &[]);
        render_pass.set_bind_group(1, &engine.camera.camera_bind_group, &[]);

        render_pass.draw_indexed(0..self.mesh.num_indicies, 0, 0..1);
    }
    
    fn finish_rendering(&self, output: wgpu::SurfaceTexture, encoder: wgpu::CommandEncoder, drivers: &device_drivers::Drivers) {
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

    fn init_render_pass<'a>(&self, output: &wgpu::SurfaceTexture, encoder: &'a mut wgpu::CommandEncoder) -> RenderPass<'a> {
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
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        return render_pass;
    }

}
