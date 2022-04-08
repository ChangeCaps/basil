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
	w_normal: vec3<f32>;
	[[location(1)]]
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

	let normal = uniforms.transform * vec4<f32>(in.normal, 0.0);
	out.w_normal = normal.xyz;

    out.uv = in.uv;

    return out;
}

[[group(1), binding(0)]]
var base_color_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var base_color_sampler: sampler;

[[stage(fragment)]]
fn frag(in: VertexOutput, [[builtin(front_facing)]] front_facing: bool) -> [[location(0)]] vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, in.uv).rgb;

	var normal = in.w_normal;
	
	if (!front_facing) {
		normal = -normal;
	}

	let light_direction = vec3<f32>(0.0, 1.0, 1.0);
	let diffuse = max(dot(normal, normalize(light_direction)), 0.0);

	var light = vec3<f32>(0.001);	

	light = light + diffuse * vec3<f32>(0.9, 0.8, 0.7) * 4.0;

	var color = base_color * light;
	color = color / (color + vec3<f32>(1.0));
	color = pow(color, vec3<f32>(1.0/2.2)); 

    return vec4<f32>(color, 1.0);
}