#version 450
/**
 * A debug shader only providing the position of the vertex.
 */

#include <mc_uniforms.glsl>

layout(location=0) in vec3 position;

void main() {
    gl_Position = mc_projection_matrix() * (mc_model_view_matrix() * vec4(position, 1.0));
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}