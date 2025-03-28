@group(0) @binding(0) var<uniform> viewport: vec2<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4(
            input.position.x * 2.0 / viewport.x - 1.0,
            input.position.y * -2.0 / viewport.y + 1.0,
            0.0,
            1.0
        ),
        input.color
    );
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
