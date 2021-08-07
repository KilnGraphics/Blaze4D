package me.hydos.rosella.example.source;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.display.GlfwWindow;
import me.hydos.rosella.file.model.GlbModelLoader;
import me.hydos.rosella.file.model.GlbRenderObject;
import me.hydos.rosella.render.PolygonMode;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.GuiRenderObject;
import me.hydos.rosella.render.pipeline.Pipeline;
import me.hydos.rosella.render.pipeline.state.StateInfo;
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

import java.util.List;

/**
 * Test which contains source related stuff
 */
public class GenericSourceTest {
    public static final GlfwWindow window;
    public static final Rosella rosella;

    public static final int WIDTH = 1280;

    public static final int TOP = 720;

    public static final Matrix4f viewMatrix = new Matrix4f();
    public static final Matrix4f projectionMatrix = new Matrix4f().ortho(-WIDTH / 2f, WIDTH / 2f, -TOP / 2f, TOP / 2f, -100, 1000);

    public static Material menuBackground;
    public static Material portalLogo;

    public static ShaderProgram basicShader;
    public static ShaderProgram guiShader;

    public static List<GlbRenderObject> spy;
    public static List<GlbRenderObject> spy2;

    public static List<GlbRenderObject> engineer;
    public static List<GlbRenderObject> engineer2;
    public static List<GlbRenderObject> engineer3;

    public static void main(String[] args) {
        Configuration.ASSIMP_LIBRARY_NAME.set("/home/haydenv/IdeaProjects/hYdos/rosella/libassimp.so"); //FIXME: LWJGL bad. LWJGL 4 when https://github.com/LWJGL/lwjgl3/issues/642
        loadShaders();
        loadMaterials();
        setupMainMenuScene();
        rosella.renderer.rebuildCommandBuffers(rosella.renderer.mainRenderPass, (SimpleObjectManager) rosella.objectManager);
//        rosella.renderer.queueRecreateSwapchain(); FIXME: # C  [libVkLayer_khronos_validation.so+0xe16204]  CoreChecks::ValidateMemoryIsBoundToBuffer(BUFFER_STATE const*, char const*, char const*) const+0x14

        window.startAutomaticLoop(rosella, () -> {
            for (GlbRenderObject glbRenderObject : engineer) {
                glbRenderObject.modelMatrix.rotateAffineYXZ(0, 0.003f, 0);
            }

            for (GlbRenderObject glbRenderObject : engineer2) {
                glbRenderObject.modelMatrix.rotateAffineYXZ(0, 0, 0.001f);
            }

            for (GlbRenderObject glbRenderObject : engineer3) {
                glbRenderObject.modelMatrix.rotateAffineYXZ(0, 0, -0.001f);
            }

            for (GlbRenderObject glbRenderObject : spy) {
                glbRenderObject.modelMatrix.rotateAffineYXZ(0, -0.0005f, -0.001f);
            }

            for (GlbRenderObject glbRenderObject : spy2) {
                glbRenderObject.modelMatrix.rotateAffineYXZ(0, -0.0005f, 0.001f);
            }
            return true;
        });
    }

    private static void setupMainMenuScene() {
        rosella.objectManager.addObject(
                new GuiRenderObject(menuBackground, -1f, new Vector3f(0, 0, 0), WIDTH, -TOP, viewMatrix, projectionMatrix)
        );

        rosella.objectManager.addObject(
                new GuiRenderObject(portalLogo, -0.9f, new Vector3f(0, 0, 0), WIDTH / 4f, -TOP / 8f, -1f, -2.6f, viewMatrix, projectionMatrix)
        );

        GlbModelLoader.NodeSelector basicTf3Nodes = (name) -> name.startsWith("lod_0_") && !name.contains("glove");
        GlbModelLoader.NodeSelector spyNodes = (name) -> true;
        engineer = GlbModelLoader.createGlbRenderObject(rosella, Global.INSTANCE.ensureResource(new Identifier("example", "models/engineer.glb")), basicShader, basicTf3Nodes, viewMatrix, projectionMatrix);
        engineer2 = GlbModelLoader.createGlbRenderObject(rosella, Global.INSTANCE.ensureResource(new Identifier("example", "models/engineer.glb")), basicShader, basicTf3Nodes, viewMatrix, projectionMatrix);
        engineer3 = GlbModelLoader.createGlbRenderObject(rosella, Global.INSTANCE.ensureResource(new Identifier("example", "models/engineer.glb")), basicShader, basicTf3Nodes, viewMatrix, projectionMatrix);
        spy = GlbModelLoader.createGlbRenderObject(rosella, Global.INSTANCE.ensureResource(new Identifier("example", "models/spy.glb")), basicShader, spyNodes, viewMatrix, projectionMatrix);
        spy2 = GlbModelLoader.createGlbRenderObject(rosella, Global.INSTANCE.ensureResource(new Identifier("example", "models/spy.glb")), basicShader, spyNodes, viewMatrix, projectionMatrix);

        for (GlbRenderObject subModel : engineer) {
            subModel.modelMatrix.scale(10, 10, 10);
            subModel.modelMatrix.translate(0, 36, 0);
            subModel.modelMatrix.rotateAffineXYZ(-90, 0, 0);
            rosella.objectManager.addObject(subModel);
        }

        for (GlbRenderObject subModel : engineer2) {
            subModel.modelMatrix.scale(10, 10, 10);
            subModel.modelMatrix.translate(40, 36, 0);
            subModel.modelMatrix.rotateAffineXYZ(-90, 0, 0);
            rosella.objectManager.addObject(subModel);
        }

        for (GlbRenderObject subModel : engineer3) {
            subModel.modelMatrix.scale(10, 10, 10);
            subModel.modelMatrix.translate(-40, 36, 0);
            subModel.modelMatrix.rotateAffineXYZ(-90, 0, 0);
            rosella.objectManager.addObject(subModel);
        }

        for (GlbRenderObject subModel : spy) {
            subModel.modelMatrix.scale(10, 10, 10);
            subModel.modelMatrix.translate(20, 36, 0);
            subModel.modelMatrix.rotateAffineXYZ(-90, 0, 0);
            rosella.objectManager.addObject(subModel);
        }

        for (GlbRenderObject subModel : spy2) {
            subModel.modelMatrix.scale(10, 10, 10);
            subModel.modelMatrix.translate(-20, 36, 0);
            subModel.modelMatrix.rotateAffineXYZ(-90, 0, 0);
            rosella.objectManager.addObject(subModel);
        }
    }

    private static void loadMaterials() {
        menuBackground = new Material(
                ((SimpleObjectManager) rosella.objectManager).pipelineManager.registerPipeline(
                        new Pipeline(
                                rosella.renderer.mainRenderPass, // TODO: fix renderpasses being gross af
                                guiShader,
                                Topology.TRIANGLES,
                                PolygonMode.FILL,
                                VertexFormats.POSITION_COLOR3f_UV0,
                                StateInfo.DEFAULT_GUI
                        )
                ),
                ImmutableTextureMap.builder()
                        .entry("texSampler", loadTexture(
                                VK10.VK_FORMAT_R8G8B8A8_UNORM, // TODO: maybe make this srgb
                                new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT),
                                Global.INSTANCE.ensureResource(new Identifier("example", "textures/background/background01.png"))
                        ))
                        .build()
        );

        portalLogo = new Material(
                ((SimpleObjectManager) rosella.objectManager).pipelineManager.registerPipeline(
                        new Pipeline(
                                rosella.renderer.mainRenderPass, // TODO: fix renderpasses being gross af
                                guiShader,
                                Topology.TRIANGLES,
                                PolygonMode.FILL,
                                VertexFormats.POSITION_COLOR3f_UV0,
                                StateInfo.DEFAULT_GUI
                        )
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
                        1024,
                        RawShaderProgram.PoolUboInfo.INSTANCE,
                        new RawShaderProgram.PoolSamplerInfo(-1, "texSampler")
                )
        );

        guiShader = rosella.objectManager.addShader(
                new RawShaderProgram(
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.v.glsl")),
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.f.glsl")),
                        rosella.common.device,
                        rosella.common.memory,
                        1024,
                        RawShaderProgram.PoolUboInfo.INSTANCE,
                        new RawShaderProgram.PoolSamplerInfo(-1, "texSampler")
                )
        );
    }

    static {
        try {
            System.loadLibrary("renderdoc");
        } catch (UnsatisfiedLinkError e) {
            System.out.println("Failed to load renderdoc.");
        }

        window = new GlfwWindow(WIDTH, TOP, "Portal / Team Fortress 3: Java Edition", true);
        rosella = new Rosella(window, "Portal 3", true);
    }
}
