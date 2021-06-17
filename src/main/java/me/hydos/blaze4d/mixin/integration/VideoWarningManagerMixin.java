package me.hydos.blaze4d.mixin.integration;

import net.minecraft.client.resource.VideoWarningManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;
import java.util.regex.Pattern;

@Mixin(VideoWarningManager.WarningPatternLoader.class)
public class VideoWarningManagerMixin {

    @Inject(method = "buildWarning(Ljava/util/List;Ljava/lang/String;)Ljava/lang/String;", at = @At("HEAD"), cancellable = true)
    private static void whatWarningsAreYouTalkinAbout(List<Pattern> warningPattern, String info, CallbackInfoReturnable<String> cir) {
        System.out.println(info);
        cir.setReturnValue("warning");
    }
}
