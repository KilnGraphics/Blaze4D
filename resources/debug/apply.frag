#version 450

layout(input_attachment_index=0, set=0, binding=0) uniform subpassInput overlay;

layout(location=0) out vec4 out_color;

void main() {
    out_color = subpassLoad(overlay);
    //out_color = vec4(1.0, 1.0, 1.0, 1.0);
}