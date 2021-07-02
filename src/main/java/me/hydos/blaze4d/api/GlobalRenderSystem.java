package me.hydos.blaze4d.api;

import java.util.Map;
import java.util.Set;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.api.vertex.ObjectInfo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import net.minecraft.util.math.Vec3f;

/**
 * Used to make bits of the code easier to manage.
 */
public class GlobalRenderSystem {
    // Shader Fields
    public static final Map<Integer, ShaderContext> SHADER_MAP = new Int2ObjectOpenHashMap<>();
    public static final Map<Integer, RawShaderProgram> SHADER_PROGRAM_MAP = new Int2ObjectOpenHashMap<>();
    public static final int DEFAULT_MAX_OBJECTS = 8092;
    public static String programErrorLog = "none";
    public static int nextShaderId = 1; // Minecraft is a special snowflake and needs shader's to start at 1
    public static int nextShaderProgramId = 1; // Same reason as above

    // Frame/Drawing Fields
    public static Set<ConsumerRenderObject> currentFrameObjects = new ObjectOpenHashSet<>();

    // Active Fields
    public static int[] boundTextureIds = new int[] {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1}; // TODO: generate an identifier instead of using int id, or switch everything over to ints
    public static int activeTexture = 0;

    public static ShaderProgram activeShader;

    // Uniforms FIXME FIXME FIXME: to add support for custom uniforms and add support for mods like iris & lambdynamic lights, we need to do this
    public static Matrix4f projectionMatrix = new Matrix4f();
    public static Matrix4f modelViewMatrix = new Matrix4f();
    public static Vector3f chunkOffset = new Vector3f();
    public static Vec3f shaderLightDirections0 = new Vec3f();
    public static Vec3f shaderLightDirections1 = new Vec3f();

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
    public static void render() {
        Blaze4D.rosella.getRenderObjects().clear();
        if (currentFrameObjects.size() < 2000) {
            for (ConsumerRenderObject renderObject : currentFrameObjects) {
                Blaze4D.rosella.addToScene(renderObject);
            }
        } else {
            Blaze4D.LOGGER.warn("Skipped a frame");
        }


        Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);

        Blaze4D.window.forceMainLoop();
        Blaze4D.rosella.getRenderer().render(Blaze4D.rosella);

//        currentFrameObjects.forEach(consumerRenderObject -> consumerRenderObject.free(Blaze4D.rosella.getMemory(), Blaze4D.rosella.getDevice()));
//        currentFrameObjects.clear();
    }

    public static void uploadObject(ObjectInfo objectInfo, Rosella rosella) {
        ConsumerRenderObject renderObject = new ConsumerRenderObject(objectInfo, rosella);
        currentFrameObjects.add(renderObject);
    }
}
