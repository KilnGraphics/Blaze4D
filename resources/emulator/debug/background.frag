#version 450

layout(input_attachment_index=0, set=0, binding=0) uniform subpassInput rendered;

layout(location=0) in vec2 in_pixel_coord;

layout(location=0) out vec4 out_color;

const float BASE_VALUE[2] = float[](0.2, 0.4);
const float OFFSET_VALUE[2] = float[](0.0, -0.1);

vec3 generate_bg() {
    int x = int(round(in_pixel_coord.x));
    int y = int(round(in_pixel_coord.y));
    float base = BASE_VALUE[((x / 200) + (y / 200)) % 2];
    float offset = OFFSET_VALUE[((x / 20) + (y / 20)) % 2];

    return vec3(base + offset);
}

void main() {
    vec4 in_color = subpassLoad(rendered);

    float alpha = in_color.a;

    out_color = vec4(((1.0 - alpha) * generate_bg()) + (alpha * in_color.rgb), 1.0);
}