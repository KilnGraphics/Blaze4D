package me.hydos.rosella.render.texture;

import it.unimi.dsi.fastutil.ints.Int2ObjectMap;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.ints.IntArrayPriorityQueue;
import it.unimi.dsi.fastutil.ints.IntPriorityQueue;
import it.unimi.dsi.fastutil.ints.IntPriorityQueues;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.util.VkUtils;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.vulkan.VK10;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

public class TextureManager {

    public static final int BLANK_TEXTURE_ID = 0;
    public static Texture BLANK_TEXTURE;

    private final VkCommon common;

    private final Int2ObjectMap<Texture> textureMap = new Int2ObjectOpenHashMap<>();
    private final Map<SamplerCreateInfo, Map<String, TextureSampler>> samplerCache = new HashMap<>();
    private final Set<Texture> preparedTextures = new HashSet<>();
    private final IntPriorityQueue reusableTexIds = IntPriorityQueues.synchronize(new IntArrayPriorityQueue());

    private int nextTexId = 1; // we start at 1 because 0 is reserved for the blank texture

    public TextureManager(VkCommon common) {
        this.common = common;
    }

    public final VkCommon getCommon() {
        return this.common;
    }

    public void initializeBlankTexture(Renderer renderer) {
        if (BLANK_TEXTURE == null) {
            createTexture(renderer, BLANK_TEXTURE_ID, 1, 1, VK10.VK_FORMAT_R8G8B8A8_UNORM);
            setTextureSamplerNoCache(BLANK_TEXTURE_ID, new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT));
            BLANK_TEXTURE = getTexture(BLANK_TEXTURE_ID);
            prepareTexture(renderer, BLANK_TEXTURE);
        } else {
            throw new RuntimeException("Blank texture already initialized");
        }
    }

    public int generateTextureId() {
        if (!reusableTexIds.isEmpty()) {
            return reusableTexIds.dequeueInt();
        } else {
            return nextTexId++;
        }
    }

    public void deleteTexture(int textureId) {
        Texture removedTex = this.textureMap.remove(textureId);
        removedTex.getTextureImage().free(common.device, common.memory);
        preparedTextures.remove(removedTex);
        reusableTexIds.enqueue(textureId);
    }

    public Texture getTexture(int textureId) {
        return textureMap.get(textureId);
    }

    public void createTexture(Renderer renderer, int textureId, int width, int height, int imgFormat) {
        Texture currentTexture = textureMap.get(textureId);
        if (currentTexture != null) {
            if (currentTexture.getImageFormat() != imgFormat || currentTexture.getWidth() != width || currentTexture.getHeight() != height) {
                currentTexture.getTextureImage().free(common.device, common.memory);
            } else {
                // we can use the old texture if it satisfies the requirements
                return;
            }
        }
        TextureImage textureImage = VkUtils.createTextureImage(renderer, common.memory, common.device, width, height, imgFormat);
        textureImage.setView(VkUtils.createTextureImageView(common.device, imgFormat, textureImage.pointer()));
        textureMap.put(textureId, new Texture(imgFormat, width, height, textureImage, VK10.VK_NULL_HANDLE));
    }

    // TODO: figure out how to use binding ids instead of names
    public void setTextureSampler(int textureId, String samplerName, SamplerCreateInfo samplerCreateInfo) {
        Map<String, TextureSampler> textureNoMap = samplerCache.computeIfAbsent(samplerCreateInfo, s -> new HashMap<>());
        TextureSampler textureSampler = textureNoMap.computeIfAbsent(samplerName, t -> new TextureSampler(samplerCreateInfo, common.device));
        textureMap.get(textureId).setTextureSampler(textureSampler.getPointer());
    }

    public void setTextureSamplerNoCache(int textureId, SamplerCreateInfo samplerCreateInfo) {
        TextureSampler textureSampler = new TextureSampler(samplerCreateInfo, common.device);
        textureMap.get(textureId).setTextureSampler(textureSampler.getPointer());
    }

    public void drawToExistingTexture(Renderer renderer, int textureId, UploadableImage image, ImageRegion srcRegion, ImageRegion dstRegion) {
        Texture texture = getTexture(textureId);
        if (preparedTextures.contains(texture)) {
            VkUtils.transitionImageLayout(
                    renderer,
                    common.device,
                    renderer.depthBuffer,
                    texture.getTextureImage().pointer(),
                    texture.getImageFormat(),
                    VK10.VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
                    VK10.VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
            );
            preparedTextures.remove(texture);
        }

        VkUtils.copyToTexture(
                renderer,
                common.device,
                common.memory,
                image,
                srcRegion,
                dstRegion,
                texture
        );
    }

    public void drawToExistingTexture(Renderer renderer, int textureId, UploadableImage image) {
        ImageRegion region = new ImageRegion(image.getWidth(), image.getHeight(), 0, 0);
        drawToExistingTexture(renderer, textureId, image, region, region);
    }

    public void prepareTexture(Renderer renderer, Texture texture) {
        if (!preparedTextures.contains(texture)) {
            VkUtils.transitionImageLayout(
                    renderer,
                    common.device,
                    renderer.depthBuffer,
                    texture.getTextureImage().pointer(),
                    texture.getImageFormat(),
                    VK10.VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    VK10.VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
            );
            preparedTextures.add(texture);
        }

    }
}
