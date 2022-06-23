#version 450

vec2 positions[4] = vec2[](
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);

vec2 pixel_coords[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);
layout(constant_id=0) const float FRAMEBUFFER_WIDTH = 1.0;
layout(constant_id=1) const float FRAMEBUFFER_HEIGHT = 1.0;

layout(location=0) out vec2 out_pixel_coord;

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);

    out_pixel_coord = vec2(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT) * pixel_coords[gl_VertexIndex];
}