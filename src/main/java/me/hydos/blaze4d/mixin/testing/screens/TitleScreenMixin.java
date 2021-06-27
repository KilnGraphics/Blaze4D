package me.hydos.blaze4d.mixin.testing.screens;

import net.minecraft.client.gui.DrawableHelper;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.util.math.MatrixStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TitleScreen.class)
public class TitleScreenMixin extends DrawableHelper {

    @Inject(method = "render", at = @At("HEAD"), cancellable = true)
    private void rectNoTexTest(MatrixStack matrices, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        ci.cancel();

        fill(matrices, 20, 20, 50, 50, 0xFFFFFFFF);
    }
}
