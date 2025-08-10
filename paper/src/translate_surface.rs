
use sdl3::video::Window;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

// contains the unsafe impl as much as possible by putting it in this module

pub struct SyncWindow(pub std::sync::Arc<Window>);

unsafe impl Send for SyncWindow {}
unsafe impl Sync for SyncWindow {}

impl HasWindowHandle for SyncWindow {
  fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
    self.0.window_handle()
  }
}

impl HasDisplayHandle for SyncWindow {
  fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
    self.0.display_handle()
  }
}

pub fn create_surface<'a>(
  instance: &wgpu::Instance,
  window: std::sync::Arc<Window>,
) -> Result<wgpu::Surface<'a>, String> {
  instance
    .create_surface(SyncWindow(window.clone()))
    .map_err(|err| err.to_string())
}