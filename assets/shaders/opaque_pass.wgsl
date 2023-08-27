// Vertex shader

const shadow_depth_texture_size: f32 = 2048.0;
const shadow_comparison_bias = 0.007;

struct Camera {
    view_proj: mat4x4<f32>,
}

struct Sun {
    view_proj: mat4x4<f32>,
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
    @location(0) shadow_position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>
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
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.shadow_position = sun.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.shadow_position = vec4<f32>(
        out.shadow_position.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5),
        out.shadow_position.z,
        out.shadow_position.w
    );
    out.normal = normalize((model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv = vertex.uv;
    return out;
}

// Fragment shader

@group(1)@binding(1)
var depth_sampler: sampler_comparison;
@group(1) @binding(2)
var shadow_depth_texture: texture_depth_2d;

const albedo = vec3<f32>(0.9, 0.9, 0.9);
const ambient_factor = 0.2;

@group(2)@binding(0)
var texture_sampler: sampler;
@group(2) @binding(1)
var color_texture: texture_2d<f32>;


@fragment
fn fs_main(
    in: Fragment
) -> @location(0) vec4<f32> {
    return textureSample(color_texture, texture_sampler, in.uv);
    
    
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

    // let lambert_factor = max(dot(normalize(vec3<f32>(2.0, 1.0, 1.0)), in.normal), 0.0);
    // let lighting_factor = min(ambient_factor + visibility * lambert_factor, 1.0);

    // return vec4(lighting_factor * albedo, 1.0);
}
