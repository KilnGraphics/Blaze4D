#version 450

layout(location=0) in vec3 position;

layout(set=0, binding=0) uniform UniformMat {
    mat4 world_ndc;
} umat;

layout(push_constant) uniform Constants {
    mat4 model_world;
} pmat;

void main() {
    gl_Position = umat.world_ndc * (pmat.model_world * vec4(position, 1.0));
}