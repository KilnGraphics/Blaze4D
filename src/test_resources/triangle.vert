#version 450

// Things Specified in the VertexFormat. In this case just the Position of each vertex. TODO: use these values instead of the hardcoded ones
layout(location = 0) in vec3 inPos;

void main() {
    const vec3 positions[3] = vec3[3](
        vec3(1.f, 1.f, 0.0f),
        vec3(-1.f, 1.f, 0.0f),
        vec3(0.f, -1.f, 0.0f)
    );

    // Tell the gpu to put a vertex at this location in GL Space. I probably could of worded that better.
    gl_Position = vec4(positions[gl_VertexIndex], 1.0f);
}