#version 450

layout(location=0) in vec2 in_uv;

layout(location=0) out vec4 color;

void main() {
    color = vec4(in_uv, 0.0, 1.0);
}