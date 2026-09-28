#version 450

layout(set = 0, binding = 0) uniform SCShaderUniforms {
    vec2 resolution;
    float time;
    float amount;
    float source_mix;
    float feedback;
    vec2 padding;
    vec4 mouse;
    float time_delta;
    float frame;
    vec2 compat_padding;
    vec4 date;
} sc;

layout(location = 0) in vec2 sc_uv;
layout(location = 0) out vec4 scshader_color;

void main() {
    float pulse = 0.5 + 0.5 * sin(sc.time + sc.amount * 6.2831853);
    scshader_color = vec4(sc_uv.x, sc_uv.y, pulse, 1.0);
}
