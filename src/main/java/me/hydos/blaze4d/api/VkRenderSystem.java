package me.hydos.blaze4d.api;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.util.Map;

public class VkRenderSystem {
    public static final Logger LOGGER = LogManager.getLogger("VkRenderSystem");
    public static final boolean DEBUG = true;
    public static final Map<Integer, ShaderContext> SHADER_MAP = new Int2ObjectOpenHashMap<>();
    public static final Map<Integer, RawShaderProgram> SHADER_PROGRAM_MAP = new Int2ObjectOpenHashMap<>();
    public static final int DEFAULT_MAX_OBJECTS = 1024;
    public static String programErrorLog = "none";
    public static int nextShaderId = 1; // Minecraft is a special snowflake and needs shader's to start at 1
    public static int nextShaderProgramId = 1; // Same reason as above

    public static net.minecraft.util.Identifier boundTexture = new net.minecraft.util.Identifier("minecraft", "empty");
    public static ShaderProgram activeShader;

    /**
     * @param glId the glId
     * @return a identifier which can be used instead of a glId
     */
    public static Identifier generateId(int glId) {
        return new Identifier("blaze4d", "gl_" + glId);
    }

    /**
     * Used for debugging stuff
     *
     * @param msg the message to be sent when debugging
     */
    public static void debug(Object msg) {
        if (VkRenderSystem.DEBUG) {
            System.out.println(msg);
        }
    }
}
