
struct CameraUniform {
  view_proj: mat4x4<f32>,
};

fn get_camera_projection(model: VertexInput) -> vec4f {
  var view_proj: vec4f = camera.view_proj * vec4<f32>(model.position, 1.0);
  return view_proj;
}