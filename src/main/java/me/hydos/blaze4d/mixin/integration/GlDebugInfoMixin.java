package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlDebugInfo;
import me.hydos.blaze4d.Blaze4D;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(GlDebugInfo.class)
public class GlDebugInfoMixin {

    /**
     * @author Blaze4D
     * @reason Reroute to Rosella equivalent
     */
    @Overwrite
    public static String getVendor() {
        return tryParseVendorId(Blaze4D.rosella.common.device.properties.vendorId);
    }

    /**
     * @author Blaze4D
     * @reason Reroute to Rosella equivalent
     */
    @Overwrite
    public static String getRenderer() {
        return Blaze4D.rosella.common.device.properties.deviceName;
    }

    /**
     * @author Blaze4D
     * @reason Reroute to Rosella equivalent
     */
    @Overwrite
    public static String getVersion() {
        return "Vulkan Api Version: " + Blaze4D.rosella.common.device.properties.apiVersion;
    }

    private static String tryParseVendorId(int vendorId) {
        return switch (vendorId) {
            case 4318 -> "NVIDIA Corporation";
            default -> "Vendor unknown. Vendor ID: " + vendorId;
        };
    }
}
