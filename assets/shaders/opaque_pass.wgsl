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
    @location(2) uv: vec2<f32>,
};

struct Instance {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct Fragment {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) shadow_position: vec4<f32>,
};

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
    out.world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    out.clip_position = camera.view_proj * out.world_position;
    out.shadow_position = sun.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.normal = (model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.uv = vertex.uv;
    return out;
}

// Fragment shader

@group(1)@binding(1)
var depth_sampler: sampler_comparison;
@group(1) @binding(2)
var shadow_depth_texture: texture_depth_2d;

const PI = 3.14159;
const shadow_depth_texture_size: f32 = 2048.0;
const shadow_comparison_bias = 0.007;

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let num = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}  

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
    let albedo = textureSample(color_texture, texture_sampler, in.uv).rgb;
    let metallic_roughness = textureSample(metallic_roughness_texture, texture_sampler, in.uv);
    let metallic = metallic_roughness.r;
    let roughness = metallic_roughness.g;

    let N = normalize(in.normal);
    let V = normalize(camera.position.xyz - in.world_position.xyz);

    var F0 = vec3<f32>(0.04, 0.04, 0.04);
    F0 = mix(F0, albedo, metallic);
	           
    // reflectance equation
    var Lo = vec3(0.0);

    let L = normalize(sun.position.xyz - in.world_position.xyz);
    let H = normalize(V + L);
    let distance = length(sun.position - in.world_position);
    let attenuation = 1.0 / (distance * distance);
    let radiance = sun.position.xyz * attenuation;        
        
    // cook-torrance brdf
    let NDF = DistributionGGX(N, H, roughness);
    let G = GeometrySmith(N, V, L, roughness);
    let F = fresnelSchlick(max(dot(H, V), 0.0), F0);

    let kS = F;
    var kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    let numerator = NDF * G * F;
    let denominator = max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;  
            
    // add to outgoing radiance Lo
    let NdotL = max(dot(N, L), 0.0);
    Lo += (kD * albedo / PI + specular) * radiance * NdotL;

    let ambient = vec3(0.03) * albedo;

    // var visibility = 0.0;
    // let oneOverShadowDepthTextureSize = 1.0 / shadow_depth_texture_size;
    // for (var y = -1; y <= 1; y++) {
    //     for (var x = -1; x <= 1; x++) {
    //         let offset = vec2<f32>(vec2(x, y)) * oneOverShadowDepthTextureSize;

    //         visibility += textureSampleCompare(
    //             shadow_depth_texture,
    //             depth_sampler,
    //             in.shadow_position.xy + offset,
    //             in.shadow_position.z + shadow_comparison_bias
    //         );
    //     }
    // }
    // visibility /= 9.0;

    var color = ambient + Lo;
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
