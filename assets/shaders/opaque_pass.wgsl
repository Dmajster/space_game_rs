// Vertex shader

struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    p0: f32,
}

struct Light {
    position: vec3<f32>,
    intensity: f32,
    //
    direction: vec3<f32>,
    inner_angle: f32,
    //
    color: vec3<f32>,
    outer_angle: f32,
    //
    falloff_radius: f32,
    ty: i32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<storage, read> lights: array<Light>;

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
    out.position = (model_matrix * vec4<f32>(vertex.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.normal = (model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.tangent = (model_matrix * vec4<f32>(vertex.tangent, 0.0)).xyz;
    out.bitangent = (model_matrix * vec4<f32>(vertex.bitangent, 0.0)).xyz;
    out.uv = vertex.uv;
    return out;
}

struct Fragment {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
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

fn BRDF(
    v: vec3<f32>,
    l: vec3<f32>,
    n: vec3<f32>,
    perceptual_roughness: f32,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    let h = normalize(v + l);

    let NoV = abs(dot(n, v)) + 0.00001;
    let NoL = clamp(dot(n, l), 0.0, 1.0);
    let NoH = clamp(dot(n, h), 0.0, 1.0);
    let LoH = clamp(dot(l, h), 0.0, 1.0);

        // perceptually linear roughness to roughness (see parameterization)
    let roughness = perceptual_roughness * perceptual_roughness;
    let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color * metallic;

    let D = D_GGX(NoH, roughness);
    let F = F_Schlick(LoH, f0);
    let V = V_SmithGGXCorrelated(NoV, NoL, roughness);

    let diffuse_color = (1.0 - metallic) * base_color.rgb;
    let Fr = (D * V) * F;
    let Fd = diffuse_color * Fd_Lambert();

    return Fr + Fd;
}

fn BSDF(v: vec3<f32>, l: vec3<f32>, n: vec3<f32>, perceptual_roughness: f32, base_color: vec3<f32>, metallic: f32) -> vec3<f32> {
    return BRDF(v, l, n, perceptual_roughness, base_color, metallic); //BTDF is ignored completly
}

fn get_square_fall_off_attenuation(pos_to_light: vec3<f32>, light_inverse_radius: f32) -> f32 {
    let distance_square = dot(pos_to_light, pos_to_light);
    let factor = distance_square * light_inverse_radius * light_inverse_radius;
    let smooth_factor = max(1.0 - factor * factor, 0.0);
    return (smooth_factor * smooth_factor) / max(distance_square, 0.0004);
}

fn get_spot_angle_attenuation(
    l: vec3<f32>,
    light_dir: vec3<f32>,
    inner_angle: f32,
    outer_angle: f32
) -> f32 {
    let cos_outer = cos(outer_angle);
    let spot_scale = 1.0 / max(cos(inner_angle) - cos_outer, 0.0004);
    let spot_offset = -cos_outer * spot_scale;

    let cd = dot(normalize(-light_dir), l);
    let attenuation = clamp(cd * spot_scale + spot_offset, 0.0, 1.0);
    return attenuation * attenuation;
}

fn evaluate_point_light(
    light: Light,
    world_position: vec3<f32>,
    v: vec3<f32>,
    n: vec3<f32>,
    perceptual_roughness: f32,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    let pos_to_light = light.position - world_position;
    let l = normalize(pos_to_light);
    let  NoL = clamp(dot(n, l), 0.0, 1.0);
    var attenuation = get_square_fall_off_attenuation(pos_to_light, 1.0 / light.falloff_radius);
    let luminance = (BSDF(v, l, n, perceptual_roughness, base_color, metallic) * light.intensity * attenuation * NoL) * light.color;
    return luminance;
}

fn evaluate_spot_light(
    light: Light,
    world_position: vec3<f32>,
    v: vec3<f32>,
    n: vec3<f32>,
    perceptual_roughness: f32,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    let pos_to_light = light.position - world_position;
    let l = normalize(pos_to_light);
    let  NoL = clamp(dot(n, l), 0.0, 1.0);

    var attenuation = get_square_fall_off_attenuation(pos_to_light, 1.0 / light.falloff_radius);
    attenuation *= get_spot_angle_attenuation(l, light.direction, light.inner_angle, light.outer_angle);

    let luminance = (BSDF(v, l, n, perceptual_roughness, base_color, metallic) * light.intensity * attenuation * NoL) * light.color;
    return luminance;
}

fn evaluate_directional_light(
    light: Light,
    v: vec3<f32>,
    n: vec3<f32>,
    perceptual_roughness: f32,
    base_color: vec3<f32>,
    metallic: f32,
) -> vec3<f32> {
    let l = normalize(-light.direction);
    let NoL = clamp(dot(n, l), 0.0, 1.0);
    let illuminance = light.intensity * NoL;
    let luminance = BSDF(v, l, n, perceptual_roughness, base_color, metallic) * illuminance;
    return luminance;
}

// @group(1)@binding(1)
// var depth_sampler: sampler_comparison;
// @group(1) @binding(2)
// var shadow_depth_texture: texture_depth_2d;

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
    let base_color = (textureSample(color_texture, texture_sampler, in.uv) * material_properties.base_color_factor).xyz;
    let metallic = textureSample(metallic_roughness_texture, texture_sampler, in.uv).r * material_properties.metallic_factor ;
    let perceptual_roughness = textureSample(metallic_roughness_texture, texture_sampler, in.uv).g * material_properties.roughness_factor;

    let tbn = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    var n = textureSample(normal_texture, texture_sampler, in.uv).rgb;
    n = n * 2.0 - 1.0;
    n = normalize(tbn * n);

    let v = normalize(camera.position - in.position);
    var color = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0; i < 2; i++) {
        if lights[i].ty == 0 {
            color += evaluate_directional_light(lights[i], v, n, perceptual_roughness, base_color, metallic);
        } else if lights[i].ty == 1 {
            color += evaluate_point_light(lights[i], in.position, v, n, perceptual_roughness, base_color, metallic);
        } else {
            color += evaluate_spot_light(lights[i], in.position, v, n, perceptual_roughness, base_color, metallic);
        }
    }

    return vec4<f32>(color, 1.0);
}