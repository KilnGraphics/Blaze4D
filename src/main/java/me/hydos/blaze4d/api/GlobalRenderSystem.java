package me.hydos.blaze4d.api;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;

import java.util.List;
import java.util.Map;

/**
 * Used to make bits of the code easier to manage.
 */
public class GlobalRenderSystem {
    // Shader Fields
    public static final Map<Integer, ShaderContext> SHADER_MAP = new Int2ObjectOpenHashMap<>();
    public static final Map<Integer, RawShaderProgram> SHADER_PROGRAM_MAP = new Int2ObjectOpenHashMap<>();
    public static final int DEFAULT_MAX_OBJECTS = 1024;
    public static String programErrorLog = "none";
    public static int nextShaderId = 1; // Minecraft is a special snowflake and needs shader's to start at 1
    public static int nextShaderProgramId = 1; // Same reason as above

    // Frame/Drawing Fields
    public static List<ConsumerRenderObject> frameObjects = new ObjectArrayList<>(); // The fastest list i could find

    // Active Fields
    public static net.minecraft.util.Identifier boundTexture = new net.minecraft.util.Identifier("minecraft", "empty");
    public static ShaderProgram activeShader;

    //=================
    // Shader Methods
    //=================

    /**
     * @param glId the glId
     * @return a identifier which can be used instead of a glId
     */
    public static Identifier generateId(int glId) {
        return new Identifier("blaze4d", "gl_" + glId);
    }

    //=================
    // Frame/Drawing Methods
    //=================

    /**
     * Called when a frame is flipped. used to send all buffers to the engine to draw. Also allows for caching
     */
    public static void flipFrame() {
        if (Blaze4D.rosella.getRenderObjects().size() != 0) {
            for (Renderable renderable : Blaze4D.rosella.getRenderObjects().values()) {
                renderable.free(Blaze4D.rosella.getMemory());
            }
            Blaze4D.rosella.getRenderObjects().clear();
        }

        if (frameObjects.size() != 0) {
            Blaze4D.rosella.getRenderer().clearCommandBuffers();
        }

        for (ConsumerRenderObject renderObject : frameObjects) {
            Blaze4D.rosella.addRenderObject(renderObject, renderObject.toString());
        }

        if (frameObjects.size() != 0) {
            Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);
        }

        frameObjects.clear();
        Blaze4D.window.forceMainLoop();
    }

    public static void uploadObject(ConsumerRenderObject renderObject) {
        frameObjects.add(renderObject);
    }
}
