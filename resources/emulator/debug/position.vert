#version 450
/**
 * A debug shader only providing the position of the vertex.
 */

#include <mc_uniforms.glsl>

layout(constant_id=0) const bool has_color = false;

layout(location=0) in vec3 in_position;
layout(location=1) in vec4 in_color;

layout(location=0) out vec4 out_color;

void main() {
    gl_Position = mc_projection_matrix() * (mc_model_view_matrix() * vec4(in_position, 1.0));
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;

    if(has_color) {
        out_color = in_color;
    } else {
        out_color = vec4(0.0, 0.0, 0.0, 0.0);
    }
}