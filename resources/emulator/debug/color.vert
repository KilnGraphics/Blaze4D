#version 450
/**
 * A debug shader passing color data to the fragment shader.
 */

#include <mc_uniforms.glsl>

layout(location=0) in vec3 in_position;
layout(location=1) in vec4 in_color;

layout(location=0) out vec4 out_color;

void main() {
    gl_Position = mc_transform_position(in_position);
    out_color = in_color;
}