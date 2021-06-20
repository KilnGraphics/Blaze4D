package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.api.VkRenderSystem;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "setShaderTexture(ILnet/minecraft/util/Identifier;)V", at = @At("HEAD"))
    private static void setTexture(int i, Identifier identifier, CallbackInfo ci) {
        VkRenderSystem.boundTexture = identifier;
    }
}
