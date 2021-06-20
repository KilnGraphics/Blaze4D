package me.hydos.blaze4d.mixin.texture;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.Executor;

import me.hydos.blaze4d.api.VkRenderSystem;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = AbstractTexture.class, priority = 1001)
public abstract class AbstractTextureMixin {

    private static final Map<Integer, AbstractTexture> GL_ID_TO_TEXTURE = new HashMap<>();

    private static int nextGlTexId = 1;
    private final int glTexId = nextGlTexId++;
    private Identifier rosellaIdentifier;

    @Inject(method = "bindTexture", at = @At("HEAD"), cancellable = true)
    private void bindFake(CallbackInfo ci) {
        VkRenderSystem.boundTexture = rosellaIdentifier;
        ci.cancel();
    }

    @Inject(method = "registerTexture", at = @At("HEAD"))
    private void captureIdentifier(TextureManager textureManager, ResourceManager resourceManager, Identifier identifier, Executor executor, CallbackInfo ci) {
        this.rosellaIdentifier = identifier;
    }

    @Inject(method = "getGlId", at = @At("HEAD"), cancellable = true)
    private void getFakeId(CallbackInfoReturnable<Integer> cir) {
        GL_ID_TO_TEXTURE.computeIfAbsent(glTexId, id -> (AbstractTexture) (Object) this);
        cir.setReturnValue(glTexId);
    }
}
