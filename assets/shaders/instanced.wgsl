

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct GpuTime {
    time_secs: f32,
};

@group(2) @binding(0)
var<uniform> time: GpuTime;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    let angle = time.time_secs;
    let cos_theta = cos(angle);
    let sin_theta = sin(angle);
    let rotation_z = mat4x4<f32>(
        vec4<f32>( cos_theta, 0.0, sin_theta, 0.0),
        vec4<f32>(       0.0, 1.0, 0.0,       0.0),
        vec4<f32>(-sin_theta, 0.0, cos_theta, 0.0),
        vec4<f32>(       0.0, 0.0, 0.0,       1.0)
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * rotation_z * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
