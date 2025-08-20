use std::sync::Arc;

use crate::window_layer::translate_surface;

pub struct Drivers {
  pub surface: wgpu::Surface<'static>,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub window: Arc<sdl3::video::Window>,
  pub surface_config: wgpu::SurfaceConfiguration,
}

impl Drivers {
  async fn init_window<'a>(
    window: Arc<sdl3::video::Window>,
  ) -> (
    wgpu::Surface<'static>,
    wgpu::Device,
    wgpu::Queue,
    (u32, u32),
    wgpu::TextureFormat,
    wgpu::SurfaceCapabilities,
  ) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      backends: wgpu::Backends::PRIMARY,
      ..Default::default()
    });
    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let surface = translate_surface::create_surface(&instance, window.clone()).unwrap();

    let size = window.size();
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::default(),
          memory_hints: Default::default(),
        },
        None,
      )
      .await
      .unwrap();
    let surface_caps = surface.get_capabilities(&adapter);
    // formatted to srgb, check docs to change (please don't)
    let surface_format = surface_caps
      .formats
      .iter()
      .copied()
      .find(|f| f.is_srgb())
      .unwrap_or(surface_caps.formats[0]);

    return (surface, device, queue, size, surface_format, surface_caps);
  }

  pub async fn new(window: Arc<sdl3::video::Window>) -> Self {
    let (surface, device, queue, size, surface_format, surface_caps) =
      Self::init_window(window.clone()).await;
    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.0,  // width
      height: size.1, // height
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      // may change later, the amount of frames queued for rendering, 1 means lower latency
      desired_maximum_frame_latency: 2,
      view_formats: vec![],
    };

    Self {
      surface,
      device,
      queue,
      window,
      surface_config,
    }
  }
}
