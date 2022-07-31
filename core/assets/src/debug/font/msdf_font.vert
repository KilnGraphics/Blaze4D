#version 450

layout(location=0) in vec2 box_offset;
layout(location=1) in vec2 box_size;
layout(location=2) in vec2 atlas_offset;
layout(location=3) in vec2 atlas_size;
layout(location=4) in vec4 color;
layout(location=5) in uint atlas_index;

layout(location=7) in vec2 vertex_multiplier;

layout(push_constant) uniform instance_data {
    vec2 global_offset;
};

layout(set=0, binding=0) uniform framebuffer_data {
    vec2 framebuffer_size;
};

layout(location=0) out vec2 uv_cord;
layout(location=1) out vec4 out_color;
layout(location=2) flat out uint out_atlas_index;

void main() {
    uv_cord = atlas_offset + (vertex_multiplier * atlas_size);
    out_color = vec4(color.rgb, 1.0);
    out_atlas_index = atlas_index;

    vec2 position = (box_offset + (vertex_multiplier * box_size) + global_offset) * 50;
    position = ((position / framebuffer_size) - 0.5) * 2.0;
    gl_Position = vec4(position, 0.0, 1.0);
}