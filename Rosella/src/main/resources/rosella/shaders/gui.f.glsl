#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    bool wireframe = false;

    outColor = texture(texSampler, fragTexCoord);

    if(outColor.a < 0.05 && !wireframe) {
        discard;
    }

    if(wireframe) {
        outColor = vec4(1, 1, 1, 1);
    }
}