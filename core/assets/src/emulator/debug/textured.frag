#version 450

#include <mc_uniforms.glsl>

layout(location=1) in vec2 in_uv;

layout(location=0) out vec4 out_color;

layout(constant_id=0) const uint IMAGE_INDEX = 0;

void main() {
    out_color = mc_image(IMAGE_INDEX, in_uv);
}