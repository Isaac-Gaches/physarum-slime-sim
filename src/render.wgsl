struct Camera {
    res: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> cam: Camera;

@group(0) @binding(1)
var trail_tex: texture_2d<f32>;
@group(0) @binding(2)
var tex_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position.x,in.position.y,in.position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.clip_position.xy / cam.res;
    let colour = textureSample(trail_tex,tex_sampler,uv);

    return colour;
}
