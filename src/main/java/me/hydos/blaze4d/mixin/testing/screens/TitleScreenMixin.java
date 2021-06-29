package me.hydos.blaze4d.mixin.testing.screens;

import me.hydos.blaze4d.test.FakeWorld;
import net.minecraft.block.Blocks;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawableHelper;
import net.minecraft.client.gui.RotatingCubeMapRenderer;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.text.Text;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.MathHelper;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TitleScreen.class)
public class TitleScreenMixin extends Screen {

    @Shadow
    @Final
    private RotatingCubeMapRenderer backgroundRenderer;

    @Shadow
    private long backgroundFadeStart;

    protected TitleScreenMixin(Text title) {
        super(title);
    }

/*    @Inject(method = "render", at = @At("HEAD"), cancellable = true)
    private void rectNoTexTest(MatrixStack matrices, int mouseX, int mouseY, float delta, CallbackInfo ci) {
        ci.cancel();

        this.backgroundRenderer.render(delta, MathHelper.clamp(1.0F, 0.0F, 1.0F));

        MinecraftClient.getInstance().getBlockRenderManager().renderBlock(
                Blocks.GRASS_BLOCK.getDefaultState(),
                new BlockPos(0, 0, 0),
                new FakeWorld(),
                matrices,
                Tessellator.getInstance().getBuffer(),
                false,
                textRenderer.random
        );
    }*/
}
