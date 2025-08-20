use crate::gpu_layer::{
  device_drivers,
  geometry::{ModelVertex, VertexTrait},
  instances, raw_bindgroups, texture,
};

pub struct PipelineData {
  pub render_pipeline: wgpu::RenderPipeline,
}

impl PipelineData {
  fn get_gpu_buffers() -> Vec<wgpu::VertexBufferLayout<'static>> {
    vec![ModelVertex::desc(), instances::Instance::desc()]
  }

  const VERTEX_SHADER_MAIN: &str = "vs_main";
  const FRAGMENT_SHADER_MAIN: &str = "fs_main";

  fn init_render_pipeline(
    device: &wgpu::Device,
    shader_module: wgpu::ShaderModule,
    surface_config: &wgpu::SurfaceConfiguration,
    bindgroup_data: &raw_bindgroups::BindGroups,
  ) -> wgpu::RenderPipeline {
    let slice = &bindgroup_data.collect_slice();
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: slice,
      push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),

      vertex: wgpu::VertexState {
        module: &shader_module,
        entry_point: Some(Self::VERTEX_SHADER_MAIN),
        buffers: &Self::get_gpu_buffers(),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },

      fragment: Some(wgpu::FragmentState {
        // 3.
        module: &shader_module,
        entry_point: Some(Self::FRAGMENT_SHADER_MAIN),
        targets: &[Some(wgpu::ColorTargetState {
          // 4.
          format: surface_config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),

      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw, // 2.
        cull_mode: Some(wgpu::Face::Back),
        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
        polygon_mode: wgpu::PolygonMode::Fill,
        // Requires Features::DEPTH_CLIP_CONTROL
        unclipped_depth: false,
        // Requires Features::CONSERVATIVE_RASTERIZATION
        conservative: false,
      },

      depth_stencil: Some(wgpu::DepthStencilState {
        format: texture::DynamicTexture::DEPTH_BUFFER_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),

      multisample: wgpu::MultisampleState {
        count: 1,                         // 2.
        mask: !0,                         // 3.
        alpha_to_coverage_enabled: false, // 4.
      },

      multiview: None, // 5.
      cache: None,     // 6.
    });
    return render_pipeline;
  }

  pub async fn new(
    bindgroups: &raw_bindgroups::BindGroups,
    drivers: &device_drivers::Drivers,
  ) -> anyhow::Result<Self> {
    let shader = drivers
      .device
      .create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        // TODO: CHANGE THIS SHIT
        source: wgpu::ShaderSource::Wgsl(
          include_str!("../../../assets/shaders/sample.wgsl").into(),
        ),
      });
    let render_pipeline =
      Self::init_render_pipeline(&drivers.device, shader, &drivers.surface_config, bindgroups);

    Ok(Self { render_pipeline })
  }
}
