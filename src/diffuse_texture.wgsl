@group(0) @binding(0)
var trail_tex_in: texture_2d<f32>;
@group(0) @binding(1)
var trail_tex_out: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16,16,1)
fn cs_main(
    @builtin(global_invocation_id) gid : vec3<u32>
) {
    let size = textureDimensions(trail_tex_in);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let px = i32(gid.x);
    let py = i32(gid.y);

    let original = textureLoad(trail_tex_in, vec2<i32>(px, py), 0);

    var sum = vec4<f32>(0.0);
    for (var oy = -1; oy <= 1; oy += 1) {
        for (var ox = -1; ox <= 1; ox += 1) {
            let x = clamp(px + ox, 0, i32(size.x) - 1);
            let y = clamp(py + oy, 0, i32(size.y) - 1);

            sum += textureLoad(trail_tex_in, vec2<i32>(x, y), 0);
        }
    }

    let blurred = sum / 9.;

    let diffusion = 0.01;
    let evapouration = vec3<f32>(0.999);

    let result = mix(original, blurred,diffusion)*vec4<f32>(evapouration,1.);

    textureStore(trail_tex_out, vec2<i32>(px, py), result);
}