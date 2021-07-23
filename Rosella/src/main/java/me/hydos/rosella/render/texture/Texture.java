package me.hydos.rosella.render.texture;

import java.util.Objects;

public final class Texture {

    private final int imageFormat;
    private final int width;
    private final int height;
    private final TextureImage textureImage;
    private long textureSampler;

    public Texture(int imageFormat, int width, int height, TextureImage textureImage, long textureSampler) {
        this.imageFormat = imageFormat;
        this.width = width;
        this.height = height;
        this.textureImage = textureImage;
        this.textureSampler = textureSampler;
    }

    public int getImageFormat() {
        return imageFormat;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public TextureImage getTextureImage() {
        return textureImage;
    }

    public long getTextureSampler() {
        return textureSampler;
    }

    public void setTextureSampler(long textureSampler) {
        this.textureSampler = textureSampler;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this) return true;
        if (obj == null || obj.getClass() != this.getClass()) return false;
        var that = (Texture) obj;
        return this.imageFormat == that.imageFormat &&
                this.width == that.width &&
                this.height == that.height &&
                Objects.equals(this.textureImage, that.textureImage);
    }

    @Override
    public int hashCode() {
        return Objects.hash(imageFormat, width, height, textureImage);
    }

    @Override
    public String toString() {
        return "Texture[" +
                "imageFormat=" + imageFormat + ", " +
                "width=" + width + ", " +
                "height=" + height + ", " +
                "textureImage=" + textureImage + ']';
    }
}
