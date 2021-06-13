package me.hydos.blaze4d.api;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.render.resource.Global;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.RawShaderProgram;

/**
 * Rosella Shaders Used temporarily until i find a way to make minecraft shaders work with {@link me.hydos.rosella.Rosella}. probably some shader post processing
 */
public class Shaders {

    public static final Identifier POSITION_COLOR = register("position_color", "assets/blaze4d", "shaders/position_color");

    public static Identifier register(String name, String assetFolder, String fileLocation) {
        Identifier id = new Identifier("minecraft", name);
        RawShaderProgram shaderProgram = new RawShaderProgram(
                Global.INSTANCE.ensureResource(new Identifier(assetFolder, fileLocation + ".vsh")),
                Global.INSTANCE.ensureResource(new Identifier(assetFolder, fileLocation + ".fsh")),
                Blaze4D.rosella.getDevice(),
                Blaze4D.rosella.getMemory(),
                9000,
                RawShaderProgram.PoolObjType.UBO,
                RawShaderProgram.PoolObjType.COMBINED_IMG_SAMPLER //TODO: remove dependency on combined image sampler
        );

        Blaze4D.rosella.registerShader(
                id,
                shaderProgram
        );
        return id;
    }
}
