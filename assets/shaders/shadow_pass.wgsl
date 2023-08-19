struct Sun {
    model_view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> sun: Sun;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uvs: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = sun.model_view_projection * vec4<f32>(model.position, 1.0);
    return out;
}