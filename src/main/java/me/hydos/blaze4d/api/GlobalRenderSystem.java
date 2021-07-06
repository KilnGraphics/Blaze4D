package me.hydos.blaze4d.api;

import java.util.*;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.api.vertex.ObjectInfo;
import me.hydos.blaze4d.mixin.shader.ShaderAccessor;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.render.VertexFormat;
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
    public static int boundTextureId = -1; // TODO: generate an identifier instead of using int id, or switch everything over to ints
    public static ShaderProgram activeShader;

    // Uniforms FIXME FIXME FIXME: to add support for custom uniforms and add support for mods like iris & lambdynamic lights, we need to do this
    // TODO: Custom uniforms are complete, but support for stuff like lambdynamic lights and iris is needed
    public static Matrix4f projectionMatrix = new Matrix4f();
    public static Matrix4f modelViewMatrix = new Matrix4f();
    public static Vector3f chunkOffset = new Vector3f();
    public static Vec3f shaderLightDirections0 = new Vec3f();
    public static Vec3f shaderLightDirections1 = new Vec3f();

    // Captured Shader for more dynamic uniforms and samplers
    public static ShaderAccessor blaze4d$capturedShader = null;

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
        Blaze4D.rosella.waitForIdle();
        ((SimpleObjectManager) Blaze4D.rosella.objectManager).renderObjects.clear();
        if (currentFrameObjects.size() < 2000) {
            for (ConsumerRenderObject renderObject : currentFrameObjects) {
                Blaze4D.rosella.objectManager.addObject(renderObject);
            }
        } else {
            Blaze4D.LOGGER.warn("Skipped a frame");
        }


        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.renderPass, (SimpleObjectManager) Blaze4D.rosella.objectManager);

        Blaze4D.window.update();
        Blaze4D.rosella.renderer.render(Blaze4D.rosella);

        currentFrameObjects.forEach(consumerRenderObject -> consumerRenderObject.free(Blaze4D.rosella.memory, Blaze4D.rosella.common.device));
        currentFrameObjects.clear();
    }

    public static void uploadObject(ObjectInfo objectInfo, Rosella rosella) {
        ConsumerRenderObject renderObject = new ConsumerRenderObject(objectInfo, rosella);
        currentFrameObjects.add(renderObject);
    }

    public static void renderConsumers(Map<ConsumerCreationInfo, BufferVertexConsumer> consumers) {
        for (Map.Entry<ConsumerCreationInfo, BufferVertexConsumer> entry : consumers.entrySet()) {
            BufferVertexConsumer consumer = entry.getValue();
            List<Integer> indices = new ArrayList<>();
            ConsumerCreationInfo creationInfo = entry.getKey();

            if (creationInfo.drawMode() == VertexFormat.DrawMode.QUADS) {
                // Convert Quads to Triangle Strips
                //  0, 1, 2
                //  0, 2, 3
                //        v0_________________v1
                //         / \               /
                //        /     \           /
                //       /         \       /
                //      /             \   /
                //    v2-----------------v3

                for (int i = 0; i < consumer.getVertexCount(); i += 4) {
                    indices.add(i);
                    indices.add(1 + i);
                    indices.add(2 + i);

                    indices.add(2 + i);
                    indices.add(3 + i);
                    indices.add(i);
                }
            } else {
                for (int i = 0; i < consumer.getVertexCount(); i++) {
                    indices.add(i);
                }
            }

            if (consumer.getVertexCount() != 0) {
                ObjectInfo objectInfo = new ObjectInfo(
                        consumer,
                        creationInfo.drawMode(),
                        creationInfo.format(),
                        creationInfo.shader(),
                        creationInfo.boundTextureId(),
                        creationInfo.projMatrix(),
                        creationInfo.viewMatrix(),
                        creationInfo.chunkOffset(),
                        creationInfo.shaderLightDirections0(),
                        creationInfo.shaderLightDirections1(),
                        Collections.unmodifiableList(indices)
                );
                if (creationInfo.shader() != null) {
                    GlobalRenderSystem.uploadObject(objectInfo, Blaze4D.rosella);
                }
            }
        }
    }
}
