package graphics.kiln.blaze4d.api.shader;

import graphics.kiln.rosella.render.resource.Resource;
import graphics.kiln.rosella.render.shader.ShaderType;

/**
 * Gives us context when hooking into low level spots inside of Shaders.
 */
public class ShaderContext {
    public Resource shader;
    public int glShaderType;
    public ShaderType rosellaShaderType;
}
