package me.hydos.blaze4d.api.util;

import graphics.kiln.rosella.render.resource.Global;
import graphics.kiln.rosella.render.resource.Identifier;
import graphics.kiln.rosella.render.resource.Resource;
import graphics.kiln.rosella.render.resource.ResourceLoader;
import org.jetbrains.annotations.NotNull;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.ByteBuffer;

public record ByteArrayResource(byte[] array) implements Resource {

    @NotNull
    @Override
    public Identifier getIdentifier() {
        return Identifier.getEMPTY();
    }

    @NotNull
    @Override
    public ResourceLoader getLoader() {
        return Global.INSTANCE;
    }

    @NotNull
    @Override
    public InputStream openStream() {
        return new ByteArrayInputStream(array);
    }

    @NotNull
    @Override
    public ByteBuffer readAllBytes(boolean n) {
        if (n) {
            ByteBuffer buffer = ByteBuffer.allocateDirect(array.length);
            buffer.put(array);
            buffer.rewind();
            return buffer;
        }

        return ByteBuffer.wrap(array);
    }
}
