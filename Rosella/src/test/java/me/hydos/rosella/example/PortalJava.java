package me.hydos.rosella.example;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.display.GlfwWindow;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.GuiRenderObject;
import me.hydos.rosella.scene.object.Renderable;
import me.hydos.rosella.render.resource.Global;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.TextureFilter;
import me.hydos.rosella.render.vertex.VertexFormats;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.joml.Vector3f;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWKeyCallback;
import org.lwjgl.vulkan.VK10;

public class PortalJava {

    public static final GlfwWindow window = new GlfwWindow(1280, 720, "Portal 3: Java Edition", true);
    public static final Rosella rosella = new Rosella(window, "portal 3", true);

    public static Material menuBackground;
    public static Material portalLogo;

    public static ShaderProgram basicShader;
    public static ShaderProgram guiShader;

    public static Renderable background;

    public static final Identifier fontShader = new Identifier("rosella", "font_shader");

    public static void main(String[] args) {
        loadShaders();
        loadFonts();
        loadMaterials();
        setupMainMenuScene();
//        SoundManager.playback(Global.INSTANCE.ensureResource(background));
        doMainLoop();
    }

    private static void loadFonts() {
//        portalFont = FontHelper.INSTANCE.loadFont(Global.INSTANCE.ensureResource(new Identifier("rosella", "fonts/DIN Bold.otf")), rosella);
    }

    private static void setupMainMenuScene() {
        rosella.objectManager.addObject(
                new GuiRenderObject(menuBackground, -1f, new Vector3f(0, 0, 0), 1.5f, 1f)
        );

        rosella.objectManager.addObject(
                new GuiRenderObject(portalLogo, -0.9f, new Vector3f(0, 0, 0), 0.4f, 0.1f, -1f, -2.6f)
        );
    }

    private static void loadMaterials() {
        menuBackground = rosella.objectManager.registerMaterial(
                new Material(
                        Global.INSTANCE.ensureResource(new Identifier("example", "textures/background/background01.png")),
                        guiShader,
                        VK10.VK_FORMAT_R8G8B8A8_UNORM,
                        false,
                        Topology.TRIANGLES,
                        VertexFormats.Companion.getPOSITION_COLOR_UV(),
                        new SamplerCreateInfo(TextureFilter.NEAREST)
                )
        );

        portalLogo = rosella.objectManager.registerMaterial(
                new Material(
                        Global.INSTANCE.ensureResource(new Identifier("example", "textures/gui/portal2logo.png")),
                        guiShader,
                        VK10.VK_FORMAT_R8G8B8A8_SRGB,
                        true,
                        Topology.TRIANGLES,
                        VertexFormats.Companion.getPOSITION_COLOR_UV(),
                        new SamplerCreateInfo(TextureFilter.NEAREST)
                )
        );

        rosella.objectManager.submitMaterials();
    }

    private static void loadShaders() {
        basicShader = rosella.objectManager.addShader(
                new RawShaderProgram(
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/base.v.glsl")),
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/base.f.glsl")),
                        rosella.common.device,
                        rosella.memory,
                        10,
                        RawShaderProgram.PoolObjType.UBO,
                        RawShaderProgram.PoolObjType.SAMPLER
                )
        );

        guiShader = rosella.objectManager.addShader(
                new RawShaderProgram(
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.v.glsl")),
                        Global.INSTANCE.ensureResource(new Identifier("rosella", "shaders/gui.f.glsl")),
                        rosella.common.device,
                        rosella.memory,
                        10,
                        RawShaderProgram.PoolObjType.UBO,
                        RawShaderProgram.PoolObjType.SAMPLER
                )
        );
    }

    private static void doMainLoop() {
        rosella.renderer.rebuildCommandBuffers(rosella.renderer.renderPass, (SimpleObjectManager) rosella.objectManager);
        GLFW.glfwSetKeyCallback(window.pWindow, new GLFWKeyCallback() {
            boolean hasDelet;

            @Override
            public void invoke(long window, int key, int scancode, int action, int mods) {
                if (key == GLFW.GLFW_KEY_V && !hasDelet) {
                    hasDelet = true;
                    System.out.println("Delet");
//                    rosella.getRenderObjects().remove("portalLogo");
//                    rosella.renderer.rebuildCommandBuffers(rosella.renderer.renderPass, rosella);
                }
            }
        });

        window.startAutomaticLoop();
    }
}
