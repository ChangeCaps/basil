struct VertexInput {
	[[location(0)]]
	position: vec3<f32>;
	[[location(1)]]
	normal: vec3<f32>; 
	[[location(2)]]
	uv: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]]
	position: vec4<f32>;
	[[location(0)]]
	uv: vec2<f32>;
};

struct Uniforms {
	view_proj: mat4x4<f32>;
	transform: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vert(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let position = uniforms.view_proj * uniforms.transform * vec4<f32>(in.position, 1.0);
    out.position = position;

    out.uv = in.uv;

    return out;
}

[[group(1), binding(0)]]
var base_color_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var base_color_sampler: sampler;

[[stage(fragment)]]
fn frag(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(base_color_texture, base_color_sampler, in.uv);

    return vec4<f32>(color.xyz, 1.0);
}