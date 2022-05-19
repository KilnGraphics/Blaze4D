#version 450

layout(location=0) in vec3 position;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;

layout(push_constant) uniform Constants {
    mat4 world_ndc;
    mat4 model_world;
} pmat;

void main() {
    gl_Position = pmat.world_ndc * (pmat.model_world * vec4(position, 1.0));
    out_uv = in_uv;
}