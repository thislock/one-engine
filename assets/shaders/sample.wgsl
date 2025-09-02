
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = get_projection(model);
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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let texture_sample_data = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let normal_colors = vec4f(in.normal, 1.0);

    return texture_sample_data;
}
