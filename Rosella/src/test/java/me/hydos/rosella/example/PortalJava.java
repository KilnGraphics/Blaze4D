package me.hydos.rosella.example;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.display.GlfwWindow;
import me.hydos.rosella.render.PolygonMode;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.pipeline.PipelineCreateInfo;
import me.hydos.rosella.render.pipeline.state.StateInfo;
import me.hydos.rosella.render.model.GuiRenderObject;
import me.hydos.rosella.render.resource.Global;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.*;
import me.hydos.rosella.render.vertex.VertexFormats;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.Configuration;
import org.lwjgl.vulkan.VK10;

public class PortalJava {
    public static final GlfwWindow window;
    public static final Rosella rosella;

    public static final int WIDTH = 1280;

    public static final int HEIGHT = 720;

    static {
        Configuration.STACK_SIZE.set(2048);
        System.loadLibrary("renderdoc");
        window = new GlfwWindow(WIDTH, HEIGHT, "Portal 3: Java Edition", true);
        rosella = new Rosella(window, "Portal 3", true);
    }

    public static final Matrix4f viewMatrix = new Matrix4f().lookAt(2.0f, -40.0f, 2.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f);
    public static final Matrix4f projectionMatrix = new Matrix4f().perspective(
            (float) Math.toRadians(45.0),
            1280 / 720f,
            0.1f,
            1000.0f
    ).scale(1.0f, -1.0f, 1.0f);

    public static Material menuBackground;
    public static Material portalLogo;

    public static ShaderProgram basicShader;
    public static ShaderProgram guiShader;

    public static StateInfo defaultStateInfo = new StateInfo(
            VK10.VK_COLOR_COMPONENT_R_BIT | VK10.VK_COLOR_COMPONENT_G_BIT | VK10.VK_COLOR_COMPONENT_B_BIT | VK10.VK_COLOR_COMPONENT_A_BIT,
            true,
            false,
            0, 0, 0, 0,
            false,
            true,
            VK10.VK_BLEND_FACTOR_ONE, VK10.VK_BLEND_FACTOR_ZERO, VK10.VK_BLEND_FACTOR_ONE, VK10.VK_BLEND_FACTOR_ZERO,
            VK10.VK_BLEND_OP_ADD,
            false,
            false,
            VK10.VK_COMPARE_OP_LESS,
            false,
            VK10.VK_LOGIC_OP_COPY,
            1.0f
    );


    public static void main(String[] args) {
        loadShaders();
        loadMaterials();
        setupMainMenuScene();
        rosella.renderer.rebuildCommandBuffers(rosella.renderer.renderPass, (SimpleObjectManager) rosella.objectManager);
//        rosella.renderer.queueRecreateSwapchain(); FIXME: # C  [libVkLayer_khronos_validation.so+0xe16204]  CoreChecks::ValidateMemoryIsBoundToBuffer(BUFFER_STATE const*, char const*, char const*) const+0x14
        window.startAutomaticLoop(rosella);
    }

    private static void setupMainMenuScene() {
        rosella.objectManager.addObject(
                new GuiRenderObject(menuBackground, -1f, new Vector3f(0, 0, 0), 1.5f, 1f, viewMatrix, projectionMatrix)
        );

        rosella.objectManager.addObject(
                new GuiRenderObject(portalLogo, -0.9f, new Vector3f(0, 0, 0), 0.4f, 0.1f, -1f, -2.6f, viewMatrix, projectionMatrix)
        );
    }

    private static void loadMaterials() {
        menuBackground = rosella.objectManager.createMaterial(
                new PipelineCreateInfo(
                        rosella.renderer.renderPass, // TODO: fix renderpasses being gross af
                        guiShader,
                        Topology.TRIANGLES,
                        PolygonMode.FILL,
                        VertexFormats.POSITION_COLOR3f_UV0,
                        defaultStateInfo
                ),
                ImmutableTextureMap.builder()
                        .entry("texSampler", loadTexture(
                                VK10.VK_FORMAT_R8G8B8A8_UNORM, // TODO: maybe make this srgb
                                new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT),
                                Global.INSTANCE.ensureResource(new Identifier("example", "textures/background/background01.png"))
                        ))
                        .build()
        );

        portalLogo = rosella.objectManager.createMaterial(
                new PipelineCreateInfo(
                        rosella.renderer.renderPass, // TODO: fix renderpasses being gross af
                        guiShader,
                        Topology.TRIANGLES,
                        PolygonMode.FILL,
                        VertexFormats.POSITION_COLOR3f_UV0,
                        defaultStateInfo
                ),
                ImmutableTextureMap.builder()
                        .entry("texSampler", loadTexture(
                                VK10.VK_FORMAT_R8G8B8A8_SRGB,
                                new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT),
                                Global.INSTANCE.ensureResource(new Identifier("example", "textures/gui/portal2logo.png"))
                        ))
                        .build()
        );
    }

    private static Texture loadTexture(int vkImgFormat, SamplerCreateInfo samplerCreateInfo, Resource imageResource) {
        TextureManager textureManager = ((SimpleObjectManager) rosella.objectManager).textureManager;

        if (imageResource.equals(Resource.Empty.INSTANCE)) {
            Rosella.LOGGER.error("Resource passed to loadTexture was empty, defaulting blank texture");
            return textureManager.getTexture(TextureManager.BLANK_TEXTURE_ID);
        }

        int textureId = textureManager.generateTextureId();
        UploadableImage image = new StbiImage(imageResource, ImageFormat.fromVkFormat(vkImgFormat));
        textureManager.createTexture(
                rosella.renderer,
                textureId,
                image.getWidth(),
                image.getHeight(),
                vkImgFormat
        );
        textureManager.setTextureSampler(
                textureId,
                "texSampler",
                samplerCreateInfo
        );
        textureManager.drawToExistingTexture(rosella.renderer, textureId, image);
        return textureManager.getTexture(textureId);
    }

    private static void loadShaders() {
        basicShader = rosella.objectManager.addShader(
                new RawShaderProgram(
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/base.v.glsl")),
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/base.f.glsl")),
                        rosella.common.device,
                        rosella.common.memory,
                        10,
                        RawShaderProgram.PoolUboInfo.INSTANCE,
                        new RawShaderProgram.PoolSamplerInfo(RawShaderProgram.Companion.getBINDING_LOCATION_AUTO(), "texSampler")
                )
        );

        guiShader = rosella.objectManager.addShader(
                new RawShaderProgram(
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.v.glsl")),
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.f.glsl")),
                        rosella.common.device,
                        rosella.common.memory,
                        10,
                        RawShaderProgram.PoolUboInfo.INSTANCE,
                        new RawShaderProgram.PoolSamplerInfo(RawShaderProgram.Companion.getBINDING_LOCATION_AUTO(), "texSampler")
                )
        );
    }
}
