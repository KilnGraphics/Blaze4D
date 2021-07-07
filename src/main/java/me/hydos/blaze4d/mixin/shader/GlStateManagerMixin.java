package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftShaderProgram;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.util.ByteArrayResource;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.util.ShaderType;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.lwjgl.opengl.GL20;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.charset.StandardCharsets;
import java.util.List;

/**
 * This Mixin handles the interactions between Minecraft shaders and GL programs and passes it onto rosella
 */
@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    //========================
    //        SHADERS
    //========================

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     */
    @Overwrite
    public static int glCreateShader(int type) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        // Check last shader's type to see if they belong in the same shader
        ShaderType rosellaType = type == GL20.GL_VERTEX_SHADER ? ShaderType.VERTEX_SHADER : ShaderType.FRAGMENT_SHADER;
        ShaderContext shaderContext = new ShaderContext();
        shaderContext.glShaderType = type;
        shaderContext.rosellaShaderType = rosellaType;
        GlobalRenderSystem.SHADER_MAP.put(GlobalRenderSystem.nextShaderId, shaderContext);
        GlobalRenderSystem.nextShaderId++;
        return GlobalRenderSystem.nextShaderId - 1;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     */
    @Overwrite
    public static void glShaderSource(int shader, List<String> shaderLines) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        ShaderContext context = GlobalRenderSystem.SHADER_MAP.get(shader);
        if (context == null) {
            throw new RuntimeException("Failed to get ShaderContext. (No shader was found with id " + shader + ")");
        }

        context.shader = shaderSrcToResource(shaderLines);
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     * If something ever goes wrong, assume its our fault :(
     */
    @Overwrite
    public static String glGetShaderInfoLog(int shader, int maxLength) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        return "Internal Blaze4D Error";
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     * <p>
     * This method is really just a method to get the compilation status of a shader.
     * as long as no exceptions have been thrown, assume everything is OK
     */
    @Overwrite
    public static int glGetShaderi(int shader, int pname) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        return GL20.GL_TRUE;
    }

    //========================
    //    SHADER PROGRAMS
    //========================

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static int glCreateProgram() {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        MinecraftShaderProgram program = new MinecraftShaderProgram(
                null,
                null,
                Blaze4D.rosella.common.device,
                Blaze4D.rosella.memory,
                GlobalRenderSystem.DEFAULT_MAX_OBJECTS,
                GlobalRenderSystem.blaze4d$capturedShader.blaze4d$getUniforms(),
                GlobalRenderSystem.blaze4d$capturedShader.blaze4d$getSamplerNames());
        GlobalRenderSystem.SHADER_PROGRAM_MAP.put(GlobalRenderSystem.nextShaderProgramId, program);
        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.renderPass, (SimpleObjectManager) Blaze4D.rosella.objectManager);
        return GlobalRenderSystem.nextShaderProgramId++;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void glAttachShader(int programId, int shaderId) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        ShaderContext shader = GlobalRenderSystem.SHADER_MAP.get(shaderId);
        RawShaderProgram program = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(programId);
        if (program == null) {
            throw new RuntimeException("Shader was requested without begin registered");
        }

        if (shader.rosellaShaderType == ShaderType.VERTEX_SHADER) {
            program.setVertexShader(shader.shader);
        } else {
            program.setFragmentShader(shader.shader);
        }
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * Basically compiles the shader program
     */
    @Overwrite
    public static void glLinkProgram(int program) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        Identifier id = GlobalRenderSystem.generateId(program);
        Blaze4D.rosella.objectManager.addShader(GlobalRenderSystem.SHADER_PROGRAM_MAP.get(program));
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * Since shaders should define this in the vertex format, we shouldn't need to worry about this.
     */
    @Overwrite
    public static void _glBindAttribLocation(int program, int index, CharSequence name) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * C Documentation: "Returns a parameter from a program object"
     * It really just's lets you query things from the program like status, etc
     */
    @Overwrite
    public static int glGetProgrami(int program, int pname) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        switch (pname) {
            case GL20.GL_LINK_STATUS, GL20.GL_COMPILE_STATUS -> {
                // Since we throw exceptions instead of failing quietly, assume everything is OK
                return 1;
            }

            default -> GlobalRenderSystem.programErrorLog = "glGetProgramI is not implemented for " + pname;
        }
        return 0;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * When something errors, this is called to figure out what went wrong.
     */
    @Overwrite
    public static String glGetProgramInfoLog(int program, int maxLength) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        String lastError = GlobalRenderSystem.programErrorLog;
        GlobalRenderSystem.programErrorLog = "";
        return lastError;
    }

    //========================
    //       UTILITIES
    //========================

    /**
     * Converts a list of lines of shader source code into a {@link Resource} which can be loaded by Rosella
     *
     * @param shaderSrc the source of the shader
     * @return a readable resource for {@link me.hydos.rosella.Rosella}
     */
    private static Resource shaderSrcToResource(List<String> shaderSrc) {
        byte[] shaderBytes = String.join("\n", shaderSrc).getBytes(StandardCharsets.UTF_8);
        return new ByteArrayResource(shaderBytes);
    }
}
