#version 450
/**
 * A debug shader only providing the position of the vertex.
 */

#include <push_constants.glsl>

layout(location=0) in vec3 position;

void main() {
    gl_Position = matrices.projection * (matrices.model_view * vec4(position, 1.0));
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}