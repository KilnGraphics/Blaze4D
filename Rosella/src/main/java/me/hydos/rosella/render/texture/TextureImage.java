package me.hydos.rosella.render.texture;

import java.util.Objects;

public class TextureImage {

    private long pTextureImage;
    private long textureImageMemory;
    private long view;

    public TextureImage(long textureImage, long textureImageMemory, long view) {
        this.pTextureImage = textureImage;
        this.textureImageMemory = textureImageMemory;
        this.view = view;
    }

    public long pointer() {
        return pTextureImage;
    }

    public void setPointer(long pTextureImage) {
        this.pTextureImage = pTextureImage;
    }

    public long getTextureImageMemory() {
        return textureImageMemory;
    }

    public void setTextureImageMemory(long textureImageMemory) {
        this.textureImageMemory = textureImageMemory;
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
                "textureImage=" + pTextureImage +
                ", textureImageMemory=" + textureImageMemory +
                ", view=" + view +
                '}';
    }
}
