
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> time: GpuTime;

@group(3) @binding(0)
var<uniform> position_matrix: ObjectPosUniform;


@vertex
fn vs_main(
    model: MeshVertexInput,
) -> MeshVertexOutput {
    var out: MeshVertexOutput;
    out.clip_position = get_projection(model.position, position_matrix);
    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: MeshVertexOutput) -> @location(0) vec4<f32> {

    let texture_sample_data = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let normal_colors = vec4f(in.normal, 1.0);

    return texture_sample_data;
}
