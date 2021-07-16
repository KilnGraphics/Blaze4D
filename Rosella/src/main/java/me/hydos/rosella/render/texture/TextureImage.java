package me.hydos.rosella.render.texture;

import java.util.Objects;

public class TextureImage {

    private long textureImage;
    private long textureImageMemory;
    private long view;

    public TextureImage(long textureImage, long textureImageMemory, long view) {
        this.textureImage = textureImage;
        this.textureImageMemory = textureImageMemory;
        this.view = view;
    }

    public long getTextureImage() {
        return textureImage;
    }

    public void setTextureImage(long textureImage) {
        this.textureImage = textureImage;
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
        return textureImage == that.textureImage && textureImageMemory == that.textureImageMemory && view == that.view;
    }

    @Override
    public int hashCode() {
        return Objects.hash(textureImage, textureImageMemory, view);
    }

    @Override
    public String toString() {
        return "TextureImage{" +
                "textureImage=" + textureImage +
                ", textureImageMemory=" + textureImageMemory +
                ", view=" + view +
                '}';
    }
}
