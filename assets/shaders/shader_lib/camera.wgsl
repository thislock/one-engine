
struct CameraUniform {
  view_proj: mat4x4<f32>,
};

fn get_camera_projection(position: vec3f) -> vec4<f32> {
  var view_proj: vec4f = camera.view_proj * vec4<f32>(position, 1.0);
  return view_proj;
}

fn get_location(position: vec3f, position_matrix: ObjectPosUniform) -> vec4<f32> {
  var location = vec4<f32>(position, 1.0);
  location = position_matrix.proj * location;
  return location;
}

fn get_projection(pos: vec3f, position_matrix: ObjectPosUniform) -> vec4<f32> {
  var position: vec4f = get_location(pos, position_matrix);
  var view_proj: vec4f = camera.view_proj * position;
  return view_proj;
}