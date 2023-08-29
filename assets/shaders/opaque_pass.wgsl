// Vertex shader

struct Camera {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

struct Sun {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> sun: Sun;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,    
    @location(4) uv: vec2<f32>,
};

struct Instance {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: Vertex,
    instance: Instance,
) -> Fragment {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: Fragment;
    out.position = model_matrix * vec4<f32>(vertex.position, 1.0);
    out.clip_position = camera.view_proj * out.position;
    out.shadow_position = sun.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.normal = normalize((model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4<f32>(vertex.tangent, 0.0)).xyz);
    out.bitangent = normalize((model_matrix * vec4<f32>(vertex.bitangent, 0.0)).xyz);
    out.uv = vertex.uv;
    return out;
}

struct Fragment {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) shadow_position: vec4<f32>,
};

// Fragment shader

@group(1)@binding(1)
var depth_sampler: sampler_comparison;
@group(1) @binding(2)
var shadow_depth_texture: texture_depth_2d;

const PI = 3.14159;
const shadow_depth_texture_size: f32 = 2048.0;
const shadow_comparison_bias = 0.007;
 

@group(2)@binding(0)
var texture_sampler: sampler;
@group(2) @binding(1)
var color_texture: texture_2d<f32>;
@group(2) @binding(2)
var normal_texture: texture_2d<f32>;
@group(2) @binding(3)
var metallic_roughness_texture: texture_2d<f32>;

@fragment
fn fs_main(
    in: Fragment
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.bitangent, 1.0);
}
