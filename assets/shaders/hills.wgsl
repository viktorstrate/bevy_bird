// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_view_bind_group
#import bevy_sprite::mesh2d_struct

[[group(0), binding(0)]] var<uniform> view: View;
[[group(2), binding(0)]] var<uniform> mesh: Mesh2d;

// Bindings specified in SpecializedMaterial2d::bind_group_layout
[[group(1), binding(0)]] var texture: texture_2d<f32>;
[[group(1), binding(1)]] var texture_sampler: sampler;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    [[builtin(vertex_index)]] index: u32;
    [[location(0)]] position: vec2<f32>;
};
struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    [[builtin(position)]] clip_position: vec4<f32>;

    // Passed along to the fragment shader
    [[location(0)]] coord_position: vec2<f32>;
    [[location(1)]] height: f32;
};

/// Entry point for the vertex shader
[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var pos = vertex.position.xy;

    // Project the world position of the mesh into screen position
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(pos, 1.0, 1.0);
    // out.color = vertex.color;
    out.coord_position = pos;

    out.height = 1. - f32(vertex.index % u32(2));

    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    [[location(0)]] pos: vec2<f32>;
    [[location(1)]] height: f32;
};

/// Entry point for the fragment shader
[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let uv = vec2<f32>((in.pos.x + 500.) / 2000., (in.pos.y + 200.) / 200.);


    // Background colors
    let RED = vec3<f32>(0.67451, 0.17647, 0.07843);
    let GREEN = vec3<f32>(0.45882, 0.61569, 0.02745);

    let mask = sin(in.height * 14. + uv.x * 100.);
    let s = smoothStep(-0.1, 0.1, mask);
    var color = RED * s + (1.0-s) * GREEN;

    // Top shading
    color = color + vec3<f32>(pow((1.-in.height), 3.0) / 2.);

    // Dark border
    if (in.height < 0.008) {
        color = vec3<f32>(0.05, 0.03, 0.01);
    }

    // Texture overlay
    let texture_uv = (in.pos + 2.) % vec2<f32>(1.);
    color = color * textureSample(texture, texture_sampler, texture_uv / vec2<f32>(2.)).rgb;

    return vec4<f32>(color, 1.);
}