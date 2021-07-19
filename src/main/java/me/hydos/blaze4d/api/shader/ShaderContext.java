package me.hydos.blaze4d.api.shader;

import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.ShaderType;

/**
 * Contains Data Which is needed for hooking in at low level spots
 */
public class ShaderContext {
    public Resource shader;
    public int glShaderType;
    public ShaderType rosellaShaderType;
}
