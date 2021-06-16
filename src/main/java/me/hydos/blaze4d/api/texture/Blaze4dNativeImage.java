package me.hydos.blaze4d.api.texture;

import java.nio.ByteBuffer;

public interface Blaze4dNativeImage {

    void setChannels(int channels);

    void setPixels(ByteBuffer pixels);
}
