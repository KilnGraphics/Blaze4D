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
        return "Vulkan API Version: " + Blaze4D.rosella.common.device.properties.apiVersion;
    }

    private static String tryParseVendorId(int vendorId) {
        return switch (vendorId) {
            case 0x10DE -> "NVIDIA Corporation";
            case 0x1002 -> "AMD";
            case 0x8086 -> "INTEL";
            case 0x13B5 -> "ARM";
            case 0x1010 -> "ImgTec";
            case 0x5143 -> "Qualcomm";
            default -> "Vendor unknown. Vendor ID: " + vendorId;
        };
    }
}
