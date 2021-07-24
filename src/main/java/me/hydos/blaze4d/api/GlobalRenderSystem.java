package me.hydos.blaze4d.api;

import com.mojang.blaze3d.vertex.VertexFormat;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.*;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.mixin.shader.ShaderAccessor;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureManager;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.VK10;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.Collections;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

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
    public static Set<ConsumerRenderObject> currentFrameObjects = Collections.newSetFromMap(new Object2ObjectLinkedOpenHashMap<>()); // this is sorted

    // Active Fields
    public static final int maxTextures = 12;
    public static int[] boundTextureIds = new int[maxTextures]; // TODO: generate an identifier instead of using int id, or switch everything over to ints
    public static int activeTexture = 0;

    public static ShaderProgram activeShader;

    // TODO maybe store snapshots of this in the materials so we keep the statelessness of vulkan
    public static StateInfo currentStateInfo = new StateInfo(
            VK10.VK_COLOR_COMPONENT_R_BIT | VK10.VK_COLOR_COMPONENT_G_BIT | VK10.VK_COLOR_COMPONENT_B_BIT | VK10.VK_COLOR_COMPONENT_A_BIT,
            true,
            false,
            0, 0, 0, 0,
            false,
            false,
            VK10.VK_BLEND_FACTOR_ONE, VK10.VK_BLEND_FACTOR_ZERO, VK10.VK_BLEND_FACTOR_ONE, VK10.VK_BLEND_FACTOR_ZERO,
            VK10.VK_BLEND_OP_ADD,
            true,
            false,
            VK10.VK_COMPARE_OP_LESS,
            false,
            VK10.VK_LOGIC_OP_COPY,
            1.0f
    );

    // Uniforms FIXME FIXME FIXME: to add support for custom uniforms and add support for mods like iris & lambdynamic lights, we need to do this
    // TODO: Custom uniforms are complete, but support for stuff like lambdynamic lights and iris is needed
    public static Matrix4f projectionMatrix = new Matrix4f();
    public static Matrix4f modelViewMatrix = new Matrix4f();
    public static Vector3f chunkOffset = new Vector3f();
    public static com.mojang.math.Vector3f shaderLightDirections0 = new com.mojang.math.Vector3f();
    public static com.mojang.math.Vector3f shaderLightDirections1 = new com.mojang.math.Vector3f();

    // FIXME: dont rely on these
    public static Matrix4f tmpProjectionMatrix = new Matrix4f();
    public static Matrix4f tmpModelViewMatrix = new Matrix4f();

    // Captured Shader for more dynamic uniforms and samplers
    public static ShaderAccessor blaze4d$capturedShaderProgram = null;
    public static final int SAMPLER_NOT_BOUND = -1;
    public static Object2IntMap<String> processedSamplers = new Object2IntOpenHashMap<>();
    public static int currentSamplerBinding = 1; // we start at 1 because 0 is reserved for the main UBO

    static {
        processedSamplers.defaultReturnValue(SAMPLER_NOT_BOUND);
    }

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
        Blaze4D.rosella.common.device.waitForIdle();

        ((SimpleObjectManager) Blaze4D.rosella.objectManager).renderObjects.clear();
        for (ConsumerRenderObject renderObject : currentFrameObjects) {
            Blaze4D.rosella.objectManager.addObject(renderObject);
        }

        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.renderPass, (SimpleObjectManager) Blaze4D.rosella.objectManager);

        Blaze4D.window.update();
        Blaze4D.rosella.renderer.render();
        // FIXME: move postDraw to somewhere else
        // if we decide to have 1 bufferManager per framebuffer, do this after the framebuffer is presented
        // if we decide to have 1 bufferManager total, do this after we know ALL framebuffers have been presented
        Blaze4D.rosella.bufferManager.postDraw();

        for (ConsumerRenderObject consumerRenderObject : currentFrameObjects) {
            consumerRenderObject.free(Blaze4D.rosella.common.device, Blaze4D.rosella.common.memory);
        }
        currentFrameObjects.clear();
    }

    public static Texture[] createTextureArray() {
        Texture[] textures = new Texture[maxTextures];
        for (int i = 0; i < maxTextures; i++) {
            int texId = boundTextureIds[i];
            textures[i] = texId == TextureManager.BLANK_TEXTURE_ID ? null : ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.getTexture(texId);
        }
        return textures;
    }

    public static void uploadAsyncCreatableObject(ManagedBuffer<ByteBuffer> vertexBufferSource, ManagedBuffer<ByteBuffer> indexBufferSource,
                                    int indexCount, me.hydos.rosella.render.vertex.VertexFormat format, Topology topology,
                                    ShaderProgram shader, Texture[] textures, StateInfo stateInfo, Matrix4f projMatrix,
                                    Matrix4f modelViewMatrix, Vector3f chunkOffset, com.mojang.math.Vector3f shaderLightDirections0,
                                    com.mojang.math.Vector3f shaderLightDirections1, Rosella rosella) {

        if (shader == null) return;
        ConsumerRenderObject renderObject = new ConsumerRenderObject(
                CompletableFuture.completedFuture(new RenderInfo(rosella.bufferManager.getOrCreateVertexBuffer(vertexBufferSource), rosella.bufferManager.getOrCreateIndexBuffer(indexBufferSource), indexCount)), // TODO: designate thread pool for this maybe
                format,
                topology,
                shader,
                textures,
                stateInfo,
                projMatrix,
                modelViewMatrix,
                chunkOffset,
                shaderLightDirections0,
                shaderLightDirections1,
                rosella
        );
        currentFrameObjects.add(renderObject);
    }

    public static void uploadPreCreatedObject(RenderInfo renderInfo, me.hydos.rosella.render.vertex.VertexFormat format,
                                    Topology topology, ShaderProgram shader, Texture[] textures, StateInfo stateInfo, Matrix4f projMatrix,
                                    Matrix4f modelViewMatrix, Vector3f chunkOffset, com.mojang.math.Vector3f shaderLightDirections0,
                                    com.mojang.math.Vector3f shaderLightDirections1, Rosella rosella) {

        if (shader == null) return;
        ConsumerRenderObject renderObject = new ConsumerRenderObject(
                CompletableFuture.completedFuture(renderInfo),
                format,
                topology,
                shader,
                textures,
                stateInfo,
                projMatrix,
                modelViewMatrix,
                chunkOffset,
                shaderLightDirections0,
                shaderLightDirections1,
                rosella
        );
        currentFrameObjects.add(renderObject);
    }

    public static ObjectIntPair<ManagedBuffer<ByteBuffer>> createIndices(VertexFormat.Mode drawMode, int vertexCount) {
        IntBuffer indices;
        int indexCount;

        // TODO: try getting index buffer from minecraft (VertexBuffer and BufferBuilder)
        switch (drawMode) {
            case QUADS -> {
                // Convert Quads to Triangle Strips
                //  0, 1, 2
                //  0, 2, 3
                //        v0_________________v1
                //         / \               /
                //        /     \           /
                //       /         \       /
                //      /             \   /
                //    v2-----------------v3
                indexCount = (int) (vertexCount * 1.5);
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < vertexCount; i += 4) {
                    indices.put(i);
                    indices.put(i + 1);
                    indices.put(i + 2);
                    indices.put(i + 2);
                    indices.put(i + 3);
                    indices.put(i);
                }
            }
            case LINES -> {
                indexCount = (int) (vertexCount * 1.5);
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < vertexCount; i += 4) {
                    indices.put(i);
                    indices.put(i + 1);
                    indices.put(i + 2);
                    indices.put(i + 3);
                    indices.put(i + 2);
                    indices.put(i + 1);
                }
            }
            default -> {
                indexCount = vertexCount;
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < vertexCount; i++) {
                    indices.put(i);
                }
            }
        }

        indices.rewind();
        return new ObjectIntImmutablePair<>(new ManagedBuffer<>(MemoryUtil.memByteBuffer(indices), true), indexCount);
    }

}
