#version 450
/**
 * A debug shader passing 0 to the fragment shader.
 */

#include <mc_uniforms.glsl>

layout(location=0) in vec3 in_position;

layout(location=0) out vec4 out_color;

void main() {
    gl_Position = mc_transform_position(in_position);
    out_color = vec4(0.0, 0.0, 0.0, 0.0);
}