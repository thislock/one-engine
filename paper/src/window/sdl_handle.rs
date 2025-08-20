use std::sync::Arc;

#[allow(unused)]
pub struct SdlHandle {
  pub sdl_context: sdl3::Sdl,
  pub sdl_window: Arc<sdl3::video::Window>,
  pub event_pump: sdl3::EventPump,
}

impl SdlHandle {
  pub fn new() -> anyhow::Result<Self> {
    env_logger::init();

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
      .window("one engine demo", 800, 600)
      .position_centered()
      .resizable()
      .metal_view()
      .build()?;

    sdl_context.mouse().set_relative_mouse_mode(&window, true);

    let window = Arc::new(window);

    let event_pump = sdl_context.event_pump()?;

    Ok(Self {
      sdl_context,
      sdl_window: window,
      event_pump,
    })
  }
}
