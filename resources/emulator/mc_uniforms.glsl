/**
 * Defines all inputs to support minecrafts uniforms.
 */


layout(set=0, binding=0, std140)
uniform _McStaticUniforms {
    mat4 projection_matrix;
    vec4 fog_color;
    vec3 fog_range_and_game_time;
    uint fog_shape;
    vec2 screen_size;
} _mc_static_uniforms;

/*
layout(set=1, binding=0, std140)
uniform McSet1Binding0 {
    mat4 inverse_view_rotation_matrix;
    mat4 texture_matrix;
    vec3 light_0_direction;
    vec3 light_1_direction;
    vec4 color_modulator;
    float line_width;
} mc_set_1_binding_0;*/

layout(push_constant)
uniform _PushConstant {
    mat4 model_view_matrix;
    vec3 chunk_offset;
} _push_constant;

/*
layout(set=1, binding=1)
uniform texture2D mc_texture_0;

layout(set=1, binding=2)
uniform texture2D mc_texture_1;

layout(set=1, binding=3)
uniform texture2D mc_texture_2;*/

mat4 mc_model_view_matrix() {
    return _push_constant.model_view_matrix;
}

mat4 mc_projection_matrix() {
    return _mc_static_uniforms.projection_matrix;
}

/*
mat4 mc_inverse_view_rotation_matrix() {
    return mc_set_1_binding_0.inverse_view_rotation_matrix;
}

mat4 mc_texture_matrix() {
    return mc_set_1_binding_0.texture_matrix;
}

vec2 mc_screen_size() {
    return _mc_static_uniforms.screen_size;
}

vec4 mc_color_modulator() {
    return mc_set_1_binding_0.color_modulator;
}

vec3 mc_light_0_direction() {
    return mc_set_1_binding_0.light_0_direction;
}

vec3 mc_light_1_direction() {
    return mc_set_1_binding_0.light_1_direction;
}

vec4 mc_fog_color() {
    return _mc_static_uniforms.fog_color;
}

float mc_fog_start() {
    return _mc_static_uniforms.fog_range_and_game_time.x;
}

float mc_fog_end() {
    return _mc_static_uniforms.fog_range_and_game_time.y;
}

uint mc_fog_shape() {
    return _mc_static_uniforms.fog_shape;
}

float mc_line_width() {
    return mc_set_1_binding_0.line_width;
}

float mc_game_time() {
    return _mc_static_uniforms.fog_range_and_game_time.z;
}*/

vec3 mc_chunk_offset() {
    return _push_constant.chunk_offset;
}