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
    @location(0) shadow_position: vec4<f32>,
    @location(1) position: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) uv: vec2<f32>,
};

// Fragment shader

struct MaterialProperties {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    reflectance: f32,
    padding_1: f32,
}

const PI = 3.14159;
const shadow_depth_texture_size: f32 = 2048.0;
const shadow_comparison_bias = 0.007;
const reflectance = 0.5;

fn D_GGX(NoH: f32, a: f32) -> f32 {
    let a2 = a * a;
    let f = (NoH * a2 - NoH) * NoH + 1.0;
    return a2 / (PI * f * f);
}

fn F_Schlick(u: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3(1.0) - f0) * pow(1.0 - u, 5.0);
}

fn V_SmithGGXCorrelated(NoV: f32, NoL: f32, a: f32) -> f32 {
    let a2 = a * a;
    let GGXL = NoV * sqrt((-NoL * a2 + NoL) * NoL + a2);
    let GGXV = NoL * sqrt((-NoV * a2 + NoV) * NoV + a2);
    return 0.5 / (GGXV + GGXL);
}

fn Fd_Lambert() -> f32 {
    return 1.0 / PI;
}

@group(1)@binding(1)
var depth_sampler: sampler_comparison;
@group(1) @binding(2)
var shadow_depth_texture: texture_depth_2d;

@group(2) @binding(0)
var<uniform> material_properties: MaterialProperties;
@group(2)@binding(1)
var texture_sampler: sampler;
@group(2) @binding(2)
var color_texture: texture_2d<f32>;
@group(2) @binding(3)
var normal_texture: texture_2d<f32>;
@group(2) @binding(4)
var metallic_roughness_texture: texture_2d<f32>;

@fragment
fn fs_main(
    in: Fragment
) -> @location(0) vec4<f32> {
    let baseColor = (textureSample(color_texture, texture_sampler, in.uv) * material_properties.base_color_factor).xyz;
    let metallic = textureSample(metallic_roughness_texture, texture_sampler, in.uv).r * material_properties.metallic_factor ;
    let perceptualRoughness = textureSample(metallic_roughness_texture, texture_sampler, in.uv).g * material_properties.roughness_factor;

    let tbn = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    var n = textureSample(normal_texture, texture_sampler, in.uv).rgb;
    n = n * 2.0 - 1.0;
    n = normalize(tbn * n);

    let v = normalize(camera.position - in.position).xyz;
    let l = normalize(sun.position - in.position).xyz;

    let h = normalize(v + l);

    let NoV = abs(dot(n, v)) + 0.00001;
    let NoL = clamp(dot(n, l), 0.0, 1.0);
    let NoH = clamp(dot(n, h), 0.0, 1.0);
    let LoH = clamp(dot(l, h), 0.0, 1.0);

    // perceptually linear roughness to roughness (see parameterization)
    let roughness = perceptualRoughness * perceptualRoughness;
    let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + baseColor * metallic;

    let D = D_GGX(NoH, roughness);
    let F = F_Schlick(LoH, f0);
    let V = V_SmithGGXCorrelated(NoV, NoL, roughness);

    // specular BRDF
    let Fr = (D * V) * F;


    let diffuseColor = (1.0 - metallic) * baseColor.rgb;


    // diffuse BRDF
    let Fd = diffuseColor * Fd_Lambert();

    // apply lighting...
    
    
    return vec4<f32>(Fd + Fr, 1.0);
}