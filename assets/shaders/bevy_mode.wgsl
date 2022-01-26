// Bindings specified in SpecializedMaterial2d::bind_group_layout
[[group(1), binding(0)]] var texture: texture_2d<f32>;
[[group(1), binding(1)]] var texture_sampler: sampler;

struct FragmentInput {
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

/// Entry point for the fragment shader
[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
  return textureSample(texture, texture_sampler, in.uv);
}