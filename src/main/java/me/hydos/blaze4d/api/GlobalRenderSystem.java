package me.hydos.blaze4d.api;

import com.google.common.collect.ImmutableList;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.blaze3d.vertex.VertexFormatElement;
import it.unimi.dsi.fastutil.Pair;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.ints.IntArrayList;
import it.unimi.dsi.fastutil.ints.IntList;
import it.unimi.dsi.fastutil.ints.IntLists;
import it.unimi.dsi.fastutil.objects.*;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.api.vertex.ObjectInfo;
import me.hydos.blaze4d.mixin.shader.ShaderAccessor;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureManager;
import me.hydos.rosella.render.vertex.StoredBufferProvider;
import me.hydos.rosella.render.vertex.VertexFormats;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.VK10;

import java.nio.ByteBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Set;

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
        GlobalRenderSystem.renderConsumers();

        ((SimpleObjectManager) Blaze4D.rosella.objectManager).renderObjects.clear();
        for (ConsumerRenderObject renderObject : currentFrameObjects) {
            Blaze4D.rosella.objectManager.addObject(renderObject);
        }

        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.renderPass, (SimpleObjectManager) Blaze4D.rosella.objectManager);

        Blaze4D.window.update();
        Blaze4D.rosella.renderer.render();

        for (ConsumerRenderObject consumerRenderObject : currentFrameObjects) {
            consumerRenderObject.free(Blaze4D.rosella.common.memory, Blaze4D.rosella.common.device);
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

    public static void uploadObject(ObjectInfo objectInfo, Rosella rosella) {
        ConsumerRenderObject renderObject = new ConsumerRenderObject(objectInfo, rosella);
        currentFrameObjects.add(renderObject);
    }

    public static final List<Pair<ConsumerCreationInfo, StoredBufferProvider>> GLOBAL_BUFFER_PROVIDERS = new ObjectArrayList<>();

    public static void renderConsumers() {
        for (Pair<ConsumerCreationInfo, StoredBufferProvider> entry : GLOBAL_BUFFER_PROVIDERS) {
            StoredBufferProvider bufferProvider = entry.value();
            IntList indices = new IntArrayList();
            ConsumerCreationInfo creationInfo = entry.key();

            if (creationInfo.drawMode() == VertexFormat.Mode.QUADS) {
                // Convert Quads to Triangle Strips
                //  0, 1, 2
                //  0, 2, 3
                //        v0_________________v1
                //         / \               /
                //        /     \           /
                //       /         \       /
                //      /             \   /
                //    v2-----------------v3

                for (int i = 0; i < bufferProvider.getVertexCount(); i += 4) {
                    indices.add(i);
                    indices.add(1 + i);
                    indices.add(2 + i);

                    indices.add(2 + i);
                    indices.add(3 + i);
                    indices.add(i);
                }
            } else {
                for (int i = 0; i < bufferProvider.getVertexCount(); i++) {
                    indices.add(i);
                }
            }

            if (bufferProvider.getVertexCount() != 0) {
                ObjectInfo objectInfo = new ObjectInfo(
                        bufferProvider,
                        creationInfo.drawMode(),
                        creationInfo.format(),
                        creationInfo.shader(),
                        creationInfo.textures(),
                        creationInfo.stateInfo(),
                        creationInfo.projMatrix(),
                        creationInfo.viewMatrix(),
                        creationInfo.chunkOffset(),
                        creationInfo.shaderLightDirections0(),
                        creationInfo.shaderLightDirections1(),
                        IntLists.unmodifiable(indices)
                );
                if (creationInfo.shader() != null) {
                    GlobalRenderSystem.uploadObject(objectInfo, Blaze4D.rosella);
                }
            }
        }
        GLOBAL_BUFFER_PROVIDERS.clear();
    }

    public static void drawVertices(Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset, com.mojang.math.Vector3f shaderLightDirections0, com.mojang.math.Vector3f shaderLightDirections1, com.mojang.datafixers.util.Pair<BufferBuilder.DrawState, ByteBuffer> drawData) {
        BufferBuilder.DrawState drawInfo = drawData.getFirst(); // TODO: what does textured actually mean? i think it's something to do with index buffers
        ByteBuffer originalBuffer = drawData.getSecond();
        originalBuffer.clear();

        int vertexCount = drawInfo.vertexCount(); // getCount is actually getVertexCount and someone mapped them wrong

        if (vertexCount > 0) {
            VertexFormat format = drawInfo.format();

            StoredBufferProvider storedBufferProvider = null;
            ConsumerCreationInfo consumerCreationInfo = new ConsumerCreationInfo(drawInfo.mode(), format, GlobalRenderSystem.activeShader, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.currentStateInfo.snapshot(), projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
            int providersSize = GlobalRenderSystem.GLOBAL_BUFFER_PROVIDERS.size();

            if (providersSize > 0) {
                Pair<ConsumerCreationInfo, StoredBufferProvider> lastPair = GlobalRenderSystem.GLOBAL_BUFFER_PROVIDERS.get(providersSize - 1);
                if (lastPair.key().equals(consumerCreationInfo)) {
                    storedBufferProvider = lastPair.value();
                }
            }

            if (storedBufferProvider == null) {
                me.hydos.rosella.render.vertex.VertexFormat rosellaFormat = ConversionUtils.FORMAT_CONVERSION_MAP.get(consumerCreationInfo.format().getElements());

                if (rosellaFormat == null) {
                    ImmutableList<VertexFormatElement> mcElements = consumerCreationInfo.format().getElements();
                    me.hydos.rosella.render.vertex.VertexFormatElement[] rosellaElements = new me.hydos.rosella.render.vertex.VertexFormatElement[mcElements.size()]; // this size may change so we're not using a raw array
                    for (int i = 0; i < mcElements.size(); i++) {
                        rosellaElements[i] = ConversionUtils.ELEMENT_CONVERSION_MAP.get(mcElements.get(i));
                    }
                    rosellaFormat = VertexFormats.getFormat(rosellaElements);
                }

                storedBufferProvider = new StoredBufferProvider(rosellaFormat);
                GlobalRenderSystem.GLOBAL_BUFFER_PROVIDERS.add(new ObjectObjectImmutablePair<>(consumerCreationInfo, storedBufferProvider));
            }

            // TODO: figure out a way to accumulate these buffers to a staging buffer throughout the frame.
            // this would get rid of the need to copy the buffer here as well as the need to free the copy.
            ByteBuffer copiedBuffer = MemoryUtil.memAlloc(originalBuffer.limit());
            MemoryUtil.memCopy(originalBuffer, copiedBuffer);
            storedBufferProvider.addBuffer(copiedBuffer, 0, vertexCount, true);
        }
    }
}
