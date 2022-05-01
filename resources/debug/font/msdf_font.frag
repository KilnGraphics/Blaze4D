#version 450

layout(location=0) in vec2 uv_cord;
layout(location=1) in vec4 color;
layout(location=2) flat in uint atlas_index;

layout(set=0, binding=1) uniform sampler samp;
layout(set=0, binding=2) uniform texture2D atlases[1];

layout(constant_id=0) const float px_range = 2.0;

layout(location=0) out vec4 out_color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = texture(sampler2D(atlases[atlas_index], samp), uv_cord).rgb;
    float sd = median(msd.r, msd.g, msd.b);
    float screen_px_distance = px_range * (sd - 0.5);
    float opacity = clamp(screen_px_distance + 0.5, 0.0, 1.0);
    out_color = mix(vec4(color.rgb, 0.0), color, opacity);
}