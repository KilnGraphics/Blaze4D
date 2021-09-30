package me.hydos.blaze4d.api.util;

import graphics.kiln.rosella.render.resource.Identifier;
import graphics.kiln.rosella.render.resource.Resource;
import graphics.kiln.rosella.render.resource.ResourceLoader;
import org.jetbrains.annotations.NotNull;

import java.io.InputStream;
import java.nio.ByteBuffer;

public class EmptyResource implements Resource {

    public static final Resource EMPTY = new EmptyResource();

    @NotNull
    @Override
    public Identifier getIdentifier() {
        throw new RuntimeException("Kotlin was being a pain");
    }

    @NotNull
    @Override
    public ResourceLoader getLoader() {
        throw new RuntimeException("Kotlin was being a pain");
    }

    @NotNull
    @Override
    public InputStream openStream() {
        throw new RuntimeException("Kotlin was being a pain");
    }

    @NotNull
    @Override
    public ByteBuffer readAllBytes(boolean b) {
        throw new RuntimeException("Kotlin was being a pain");
    }
}
