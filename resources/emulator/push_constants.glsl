/**
 * The universal push constant layout used in all emulator shaders.
 */

layout(push_constant) uniform Constants {
    mat4 model_view;
    mat4 projection;
} matrices;