package graphics.kiln.blaze4d.mixin.integration;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;
import java.util.regex.Pattern;
import net.minecraft.client.renderer.GpuWarnlistManager;

@Mixin(GpuWarnlistManager.Preparations.class)
public class VideoWarningManagerMixin {

    @Inject(method = "matchAny", at = @At("HEAD"), cancellable = true)
    private static void whatWarningsAreYouTalkinAbout(List<Pattern> warningPattern, String info, CallbackInfoReturnable<String> cir) {
        cir.setReturnValue("warning");
    }
}
