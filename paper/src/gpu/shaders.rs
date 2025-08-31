use std::{collections::HashMap, sync::Arc};

use crate::{
  engine,
  gpu::{device_drivers::Drivers, geometry},
};
#[allow(unused)]
use crate::gpu::{
  device_drivers,
  geometry::{ModelVertex, VertexTrait},
  instances, raw_bindgroups, texture,
};

pub struct ShaderBundle {
  shaders: HashMap<uuid::Uuid, ShaderPipeline>,
}

impl ShaderBundle {
  pub fn new() -> Self {
    Self {
      shaders: HashMap::new(),
    }
  }

  pub fn add_shader(&mut self, shader: ShaderPipeline) -> anyhow::Result<()> {
    //let error = anyhow::Error::msg("failed to add shader to shader bundle");
    self
      .shaders
      .insert(uuid::Uuid::new_v4(), shader);

    return Ok(());
  }

  pub fn iter_shaders<'a>(&'a self) -> impl Iterator<Item = &'a ShaderPipeline> {
    self.shaders.values()
  }
}

pub struct ShaderBuilder {
  shader_file: Option<String>,
}

impl ShaderBuilder {
  pub fn from_file(filename: String) -> Self {
    Self {
      shader_file: Some(filename),
    }
  }

  pub fn build(self, drivers: &Drivers) -> Option<wgpu::ShaderModule> {
    let filename = self.shader_file?;
    let shader_string = crate::files::load_shader_str(&filename).ok()?;

    let shader = drivers
      .device
      .create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_string.into()),
      });

    return Some(shader);
  }
}

pub struct ShaderPipeline {
  pub render_pipeline: wgpu::RenderPipeline,
  pub meshes: Vec<Arc<geometry::Mesh>>,
}

impl ShaderPipeline {
  fn get_gpu_vertex_buffers<'a>() -> Vec<wgpu::VertexBufferLayout<'static>> {
    let buffers: Vec<wgpu::VertexBufferLayout<'static>> = vec![
      ModelVertex::desc(),
      //instances::Instance::desc()
    ];

    return buffers;
  }

  /* SHADER INIT */

  const VERTEX_SHADER_MAIN: &str = "vs_main";
  const FRAGMENT_SHADER_MAIN: &str = "fs_main";

  fn init_vertex_state<'a>(
    shader_module: &'a wgpu::ShaderModule,
    gpu_buffers: &'a Vec<wgpu::VertexBufferLayout<'static>>,
  ) -> wgpu::VertexState<'a> {
    wgpu::VertexState {
      module: shader_module,
      entry_point: Some(Self::VERTEX_SHADER_MAIN),
      buffers: gpu_buffers,
      compilation_options: Default::default(),
    }
  }

  fn init_fragment_state<'a>(
    shader_module: &'a wgpu::ShaderModule,
    color_target: &'a [Option<wgpu::ColorTargetState>],
  ) -> wgpu::FragmentState<'a> {
    return wgpu::FragmentState {
      // 3.
      module: shader_module,
      entry_point: Some(Self::FRAGMENT_SHADER_MAIN),
      targets: color_target,
      compilation_options: Default::default(),
    };
  }

  /* RENDERING PARAMETERS */

  fn init_gpu_primitives_state() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
      topology: wgpu::PrimitiveTopology::TriangleList, // 1.
      strip_index_format: None,
      front_face: wgpu::FrontFace::Ccw, // 2.
      // depth testing makes backface culling more expensive than it's worth
      // cull_mode: Some(wgpu::Face::Back),
      cull_mode: None,
      // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
      polygon_mode: wgpu::PolygonMode::Fill,
      // Requires Features::DEPTH_CLIP_CONTROL
      unclipped_depth: false,
      // Requires Features::CONSERVATIVE_RASTERIZATION
      conservative: false,
    }
  }

  fn init_depth_buffer() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
      format: texture::DynamicTexture::DEPTH_BUFFER_FORMAT,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
      bias: wgpu::DepthBiasState::default(),
    }
  }

  fn init_msaa() -> wgpu::MultisampleState {
    wgpu::MultisampleState {
      count: 1,                         // 2.
      mask: !0,                         // 3.
      alpha_to_coverage_enabled: false, // 4.
    }
  }

  /* SHADER PIPELINE CONSTRUCTER */
  fn init_render_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
    surface_config: &wgpu::SurfaceConfiguration,
    bindgroup_data: &raw_bindgroups::BindGroups,
  ) -> wgpu::RenderPipeline {
    let render_pipeline_layout = {
      let slice = &bindgroup_data.collect_slice();

      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: slice,
        push_constant_ranges: &[],
      })
    };

    let gpu_buffers = Self::get_gpu_vertex_buffers();
    let color_target = [Some(wgpu::ColorTargetState {
      // 4.
      format: surface_config.format,
      blend: Some(wgpu::BlendState::REPLACE),
      write_mask: wgpu::ColorWrites::ALL,
    })];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),

      vertex: Self::init_vertex_state(shader_module, &gpu_buffers),
      fragment: Some(Self::init_fragment_state(shader_module, &color_target)),

      primitive: Self::init_gpu_primitives_state(),

      depth_stencil: Some(Self::init_depth_buffer()),

      multisample: Self::init_msaa(),

      multiview: None, // 5.
      cache: None,     // 6.
    });
    return render_pipeline;
  }

  fn create_shader_module(
    drivers: &device_drivers::Drivers,
    shader_builder: ShaderBuilder,
  ) -> anyhow::Result<wgpu::ShaderModule> {
    shader_builder
      .build(drivers)
      .ok_or(anyhow::Error::msg(String::from("failed to build shader")))
  }

  pub async fn from_shader(
    bindgroups: &raw_bindgroups::BindGroups,
    drivers: &device_drivers::Drivers,
    shader_builder: ShaderBuilder,
  ) -> anyhow::Result<Self> {
    let shader = Self::create_shader_module(drivers, shader_builder)?;

    let render_pipeline = Self::init_render_pipeline(
      &drivers.device,
      &shader,
      &drivers.surface_config,
      bindgroups,
    );

    Ok(Self {
      render_pipeline,
      meshes: vec![],
    })
  }
}
