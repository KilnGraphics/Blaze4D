#version 150

uniform mat4 Projection;
uniform mat4 View;
uniform mat4 Model;
uniform float Time;
uniform vec4 Color;

in vec3 Position;
in vec3 Normal;
in vec2 UV;

out vec3 frag_Normal;
out vec2 frag_UV;
out float SomethingTime;

float make_weird_value(float Time, vec3 Position) {
    return Time * (Position.x + Position.y * Position.z);
}

float make_weird_value2(vec3 Normal) {
    return Time * (Normal.x + Normal.y * Normal.z);
}

void main() {
    gl_Position = Projection * Model * View * vec4(Position, 1.0);
    frag_Normal = Model * vec4(Normal, 1.0).xyz;
    frag_UV = UV;

    SomethingTime = make_weird_value(Time, Position) + make_weird_value2(Normal);
}