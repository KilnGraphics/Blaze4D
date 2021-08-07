package me.hydos.rosella.render.material;

import me.hydos.rosella.render.pipeline.Pipeline;
import me.hydos.rosella.render.texture.TextureMap;

/**
 * A Material has a pipeline and any attributes that aren't pipeline specific or instance specific.
 * For example, a material has a {@link TextureMap} because many instances may use the same textures,
 * but a pipeline doesn't require textures to be created.
 */
public record Material(Pipeline pipeline, TextureMap textures) {
}

