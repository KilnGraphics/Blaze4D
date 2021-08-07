package me.hydos.rosella.render.texture;

import me.hydos.rosella.device.LegacyVulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;

import java.util.Objects;

public class TextureImage implements MemoryCloseable {

    private final long pTextureImage;
    private final long textureImageMemory;
    private long view;

    public TextureImage(long textureImage, long textureImageMemory, long view) {
        this.pTextureImage = textureImage;
        this.textureImageMemory = textureImageMemory;
        this.view = view;
    }

    public long pointer() {
        return pTextureImage;
    }

    public long getTextureImageMemory() {
        return textureImageMemory;
    }

    public long getView() {
        return view;
    }

    public void setView(long view) {
        this.view = view;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        TextureImage that = (TextureImage) o;
        return pTextureImage == that.pTextureImage && textureImageMemory == that.textureImageMemory && view == that.view;
    }

    @Override
    public int hashCode() {
        return Objects.hash(pTextureImage, textureImageMemory, view);
    }

    @Override
    public String toString() {
        return "TextureImage{" +
                "pTextureImage=" + pTextureImage +
                ", textureImageMemory=" + textureImageMemory +
                ", view=" + view +
                '}';
    }

    @Override
    public void free(LegacyVulkanDevice device, Memory memory) {
        memory.freeImage(this);
    }
}
