#version 450

layout(input_attachment_index=0, set=0, binding=0) uniform subpassInput rendered;

layout(location=0) in vec2 in_pixel_coord;

layout(location=0) out vec4 out_color;

void main() {
    vec4 in_color = subpassLoad(rendered);

    out_color = in_color;
}