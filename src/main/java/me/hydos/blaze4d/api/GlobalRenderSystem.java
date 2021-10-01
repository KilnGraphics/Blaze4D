package me.hydos.blaze4d.api;

import com.mojang.blaze3d.platform.Window;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.math.Matrix4f;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectLinkedOpenHashMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.shader.VulkanUniform;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.mixin.shader.ShaderAccessor;
import graphics.kiln.rosella.Rosella;
import graphics.kiln.rosella.memory.ManagedBuffer;
import graphics.kiln.rosella.render.PolygonMode;
import graphics.kiln.rosella.render.Topology;
import graphics.kiln.rosella.render.info.RenderInfo;
import graphics.kiln.rosella.render.pipeline.state.StateInfo;
import graphics.kiln.rosella.render.resource.Identifier;
import graphics.kiln.rosella.render.shader.RawShaderProgram;
import graphics.kiln.rosella.render.shader.ShaderProgram;
import graphics.kiln.rosella.render.texture.ImmutableTextureMap;
import graphics.kiln.rosella.render.texture.Texture;
import graphics.kiln.rosella.render.texture.TextureManager;
import graphics.kiln.rosella.render.texture.TextureMap;
import graphics.kiln.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.util.Mth;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.VK10;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

import static org.lwjgl.vulkan.VK10.VK_FRONT_FACE_COUNTER_CLOCKWISE;

/**
 * Used to make bits of the code easier to manage.
 */
public class GlobalRenderSystem {

    // Misc Constants
    public static final PolygonMode DEFAULT_POLYGON_MODE = PolygonMode.FILL;

    // Supported Feature Fields
    public static boolean emulateTriangleFans;
    public static VertexFormat.Mode triFanEmulationMode = VertexFormat.Mode.TRIANGLE_STRIP;

    // Shader Fields
    public static final Map<Integer, ShaderContext> SHADER_MAP = new Int2ObjectOpenHashMap<>();
    public static final Map<Integer, RawShaderProgram> SHADER_PROGRAM_MAP = new Int2ObjectOpenHashMap<>();
    public static ShaderProgram activeShader;

    public static final int DEFAULT_MAX_OBJECTS = 8092;
    public static String programErrorLog = "none";
    public static int nextShaderId = 1; // Minecraft is a special snowflake and needs shader's to start at 1
    public static int nextShaderProgramId = 1; // Same reason as above

    // Frame/Drawing Fields
    public static Set<ConsumerRenderObject> currentFrameObjects = Collections.newSetFromMap(new Object2ObjectLinkedOpenHashMap<>()); // this is sorted

    // Texture Fields
    public static final int MAX_TEXTURES = 12;
    public static int[] boundTextureIds = new int[MAX_TEXTURES]; // TODO: generate an identifier instead of using int id, or switch everything over to ints
    public static int activeTextureSlot = 0;

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
            VK_FRONT_FACE_COUNTER_CLOCKWISE,
            PolygonMode.FILL,
            false,
            0.0f,
            0.0f,
            false,
            VK10.VK_COMPARE_OP_LESS,
            false,
            VK10.VK_LOGIC_OP_COPY,
            1.0f
    );

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
        Blaze4D.rosella.baseObjectManager.renderObjects.clear();

        for (ConsumerRenderObject renderObject : currentFrameObjects) {
            Blaze4D.rosella.common.fboManager.getPresentingFbo().objectManager.addObject(renderObject);
        }

        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.mainRenderPass);

        Blaze4D.window.update();
        Blaze4D.rosella.renderer.render();
        // FIXME: move postDraw to somewhere else
        // if we decide to have 1 bufferManager per framebuffer, do this after the framebuffer is presented
        // if we decide to have 1 bufferManager total, do this after we know ALL framebuffers have been presented
//        Blaze4D.rosella.bufferManager.postDraw();

//        for (ConsumerRenderObject consumerRenderObject : currentFrameObjects) {
//            consumerRenderObject.free(Blaze4D.rosella.common.device, Blaze4D.rosella.common.memory);
//        } FIXME TODO DEBUGGING HELP
        currentFrameObjects.clear();
    }

    public static String getSamplerNameForSlot(int slot) {
        return "Sampler" + slot;
    }

    public static TextureMap getCurrentTextureMap() {
        Map<String, Texture> map = new Object2ObjectOpenHashMap<>();
        for (int i = 0; i < MAX_TEXTURES; i++) {
            int texId = boundTextureIds[i];
            if (texId != TextureManager.BLANK_TEXTURE_ID) {
                map.put(getSamplerNameForSlot(i), Blaze4D.rosella.common.textureManager.getTexture(texId));
            }
        }
        map.put("DiffuseSampler", TextureManager.BLANK_TEXTURE); // TODO: this should be the current framebuffer
        return new ImmutableTextureMap(map);
    }

    public static void uploadAsyncCreatableObject(
            ManagedBuffer<ByteBuffer> vertexBufferSource,
            ManagedBuffer<ByteBuffer> indexBufferSource,
            int indexCount,
            ShaderProgram shader,
            Topology topology,
            graphics.kiln.rosella.render.vertex.VertexFormat format,
            StateInfo stateInfo,
            TextureMap textures,
            ByteBuffer rawUboData,
            Rosella rosella) {

        if (shader == null) return;
        ConsumerRenderObject renderObject = new ConsumerRenderObject(
                CompletableFuture.completedFuture(new RenderInfo(rosella.bufferManager.getOrCreateVertexBuffer(vertexBufferSource), rosella.bufferManager.getOrCreateIndexBuffer(indexBufferSource), indexCount)), // TODO: designate thread pool for this maybe
                shader,
                topology,
                format,
                stateInfo,
                textures,
                rawUboData,
                rosella
        );
        currentFrameObjects.add(renderObject);
    }

    public static void uploadPreCreatedObject(
            RenderInfo renderInfo,
            ShaderProgram shader,
            Topology topology,
            PolygonMode polygonMode,
            graphics.kiln.rosella.render.vertex.VertexFormat format,
            StateInfo stateInfo,
            TextureMap textures,
            ByteBuffer rawUboData,
            Rosella rosella) {

        if (shader == null) return;
        ConsumerRenderObject renderObject = new ConsumerRenderObject(
                CompletableFuture.completedFuture(renderInfo),
                shader,
                topology,
                format,
                stateInfo,
                textures,
                rawUboData,
                rosella
        );

        currentFrameObjects.add(renderObject);
    }

    public static MinecraftIndexBuffer createIndices(VertexFormat.Mode drawMode, int indexCount) {
        IntBuffer indices;
        int newIndexCount = indexCount;
        VertexFormat.Mode newMode = drawMode;

        // TODO: try getting index buffer from minecraft (VertexBuffer and BufferBuilder)
        switch (drawMode) {
            case QUADS:
                // Convert Quads to Triangle Strips
                //  0, 1, 2
                //  0, 2, 3
                //        v0_________________v1
                //         / \               /
                //        /     \           /
                //       /         \       /
                //      /             \   /
                //    v2-----------------v3
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < indexCount / 6 * 4; i += 4) {
                    indices.put(i);
                    indices.put(i + 1);
                    indices.put(i + 2);
                    indices.put(i + 2);
                    indices.put(i + 3);
                    indices.put(i);
                }
                break;
            case LINES:
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < indexCount / 6 * 4; i += 4) {
                    indices.put(i);
                    indices.put(i + 1);
                    indices.put(i + 2);
                    indices.put(i + 3);
                    indices.put(i + 2);
                    indices.put(i + 1);
                }
                break;
            case TRIANGLE_FAN:
                if (emulateTriangleFans) {
                    newMode = triFanEmulationMode;
                    switch (newMode) {
                        case TRIANGLES -> {
                            newIndexCount = (indexCount - 2) * 3;
                            indices = MemoryUtil.memAllocInt(newIndexCount);
                            for (int i = 2; i < indexCount; i++) {
                                indices.put(0);
                                indices.put(i - 1);
                                indices.put(i);
                            }
                        }
                        case TRIANGLE_STRIP -> {
                            newIndexCount = (indexCount - 1) * 2;
                            indices = MemoryUtil.memAllocInt(newIndexCount);
                            for (int i = 1; i < indexCount; i++) {
                                indices.put(0);
                                indices.put(i);
                            }
                        }
                        default -> throw new UnsupportedOperationException("Triangle fan emulation mode invalid");
                    }
                    break;
                }
            default:
                indices = MemoryUtil.memAllocInt(indexCount);
                for (int i = 0; i < indexCount; i++) {
                    indices.put(i);
                }
                break;
        }

        indices.rewind();
        return new MinecraftIndexBuffer(new ManagedBuffer<>(MemoryUtil.memByteBuffer(indices), true), newIndexCount, newMode);
    }

    public record MinecraftIndexBuffer(ManagedBuffer<ByteBuffer> rawBuffer, int newIndexCount, VertexFormat.Mode newMode) {}

    public static void updateUniforms() {
        updateUniforms(RenderSystem.getShader(), RenderSystem.getModelViewMatrix(), RenderSystem.getProjectionMatrix());
    }

    public static void updateUniforms(ShaderInstance shader, Matrix4f modelViewMatrix, Matrix4f projectionMatrix) {
        assert shader != null;
        if (shader.MODEL_VIEW_MATRIX != null) {
            shader.MODEL_VIEW_MATRIX.set(modelViewMatrix);
        }

        if (shader.PROJECTION_MATRIX != null) {
            shader.PROJECTION_MATRIX.set(projectionMatrix);
        }

        if (shader.COLOR_MODULATOR != null) {
            shader.COLOR_MODULATOR.set(RenderSystem.getShaderColor());
        }

        if (shader.FOG_START != null) {
            shader.FOG_START.set(RenderSystem.getShaderFogStart());
        }

        if (shader.FOG_END != null) {
            shader.FOG_END.set(RenderSystem.getShaderFogEnd());
        }

        if (shader.FOG_COLOR != null) {
            shader.FOG_COLOR.set(RenderSystem.getShaderFogColor());
        }

        if (shader.TEXTURE_MATRIX != null) {
            shader.TEXTURE_MATRIX.set(RenderSystem.getTextureMatrix());
        }

        if (shader.GAME_TIME != null) {
            shader.GAME_TIME.set(RenderSystem.getShaderGameTime());
        }

        if (shader.SCREEN_SIZE != null) {
            Window window = Minecraft.getInstance().getWindow();
            shader.SCREEN_SIZE.set((float)window.getWidth(), (float)window.getHeight());
        }

        if (shader.LINE_WIDTH != null) {
            shader.LINE_WIDTH.set(RenderSystem.getShaderLineWidth());
        }

        RenderSystem.setupShaderLights(shader);
    }

    public static ByteBuffer getShaderUbo(ShaderInstance shader) {
        List<Uniform> uniformList = ((ShaderAccessor) shader).blaze4d$getUniforms();
        int totalSize = Mth.roundToward(
                uniformList.stream()
                        .mapToInt(Uniform::getType)
                        .map(MinecraftUbo.UNIFORM_OFFSETS::get)
                        .reduce(0, Integer::sum),
                16
        );
        ByteBuffer mainBuffer = MemoryUtil.memAlloc(totalSize);
        long writeLocation = MemoryUtil.memAddress(mainBuffer);
        int writeOffset = 0;
        for (Uniform uniform : uniformList) {
            writeOffset = ((VulkanUniform) uniform).alignOffset(writeOffset);
            ((VulkanUniform) uniform).writeLocation(writeLocation + writeOffset);
            uniform.upload();
            writeOffset += MinecraftUbo.UNIFORM_OFFSETS.get(uniform.getType());
        }

        return mainBuffer;
    }
}
