package me.hydos.rosella.render.texture;

import me.hydos.rosella.render.renderer.Renderer;
import org.lwjgl.vulkan.VK10;

public final class BlankTextures {
    private BlankTextures() {
        // noop
    }

    private static Texture BLANK_TEXTURE;

    public static void initialize(TextureManager textureManager, Renderer renderer) {
        int normalBlankId = textureManager.generateTextureId();
        textureManager.createTexture(renderer, normalBlankId, 1, 1, VK10.VK_FORMAT_R8G8B8A8_UNORM);
        textureManager.setTextureSamplerNoCache(normalBlankId, new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT));
        BLANK_TEXTURE = textureManager.getTexture(normalBlankId);
        textureManager.prepareTexture(renderer, BLANK_TEXTURE);
    }

    public static Texture getBlankTexture() {
        return BLANK_TEXTURE;
    }
}