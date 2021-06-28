package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.Blaze4D;
import net.minecraft.client.MinecraftClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftClient.class)
public class MinecraftClientMixin {

    @Inject(method = "close", at = @At("HEAD"))
    private void shutdownRosella(CallbackInfo ci) {
        if(Blaze4D.rosella != null) {
            Blaze4D.rosella.free();
        }
    }
}
