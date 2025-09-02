
struct CameraUniform {
  view_proj: mat4x4<f32>,
};

fn get_camera_projection(model: VertexInput) -> vec4<f32> {
  var view_proj: vec4f = camera.view_proj * vec4<f32>(model.position, 1.0);
  return view_proj;
}

fn get_location(model: VertexInput) -> vec4<f32> {
  var location = vec4<f32>(model.position, 1.0);
  location = position_matrix.proj * location;
  return location;
}

fn get_projection(model: VertexInput) -> vec4<f32> {
  var position: vec4f = get_location(model);
  var view_proj: vec4f = camera.view_proj * position;
  return view_proj;
}