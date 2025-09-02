
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct GpuTime {
    time_secs: f32,
};

@group(2) @binding(0)
var<uniform> time: GpuTime;

@group(3) @binding(0)
var<uniform> position_matrix: ObjectPosUniform;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) normal: vec3f,
};