package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.shaders.AbstractUniform;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.math.Matrix4f;
import me.hydos.blaze4d.api.shader.VulkanUniform;
import me.hydos.blaze4d.api.util.ConversionUtils;
import net.minecraft.util.Mth;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.FloatBuffer;
import java.nio.IntBuffer;

@Mixin(Uniform.class)
public abstract class GlUniformMixin extends AbstractUniform implements VulkanUniform {
    @Unique
    private long writeLocation;

    @Final
    @Shadow
    private int type;

    @Final
    @Shadow
    private IntBuffer intValues;

    @Final
    @Shadow
    private FloatBuffer floatValues;

    @Final
    @Shadow
    private String name;

    @Shadow
    private boolean dirty;

    @Override
    public void writeLocation(long address) {
        writeLocation = address;
        markDirty();
    }

    @Inject(method = "upload", at = @At("HEAD"), cancellable = true)
    public void uploadToRosellaBuffer(CallbackInfo ci) {
        if (writeLocation == MemoryUtil.NULL || !dirty) {
            return;
        }

        this.dirty = false;
        if (this.type <= 3) {
            MemoryUtil.memCopy(MemoryUtil.memAddress(intValues), writeLocation, (long) (type + 1) * Integer.BYTES);
        } else if (this.type <= 7) {
            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), writeLocation, (long) (type - 3) * Float.BYTES);
        } else if (this.type <= 10) {
            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), writeLocation, (long) Mth.square(type - 6) * Float.BYTES);
        } else {
            throw new UnsupportedOperationException("Uniform has unexpected type " + type);
        }
        ci.cancel();
    }

    @Override
    public void set(Matrix4f matrix4f) {
        org.joml.Matrix4f matrix;
        if (this.name.equals("ProjMat")) {
             matrix = ConversionUtils.mcToJomlProjectionMatrix(matrix4f);
        } else {
            matrix = ConversionUtils.mcToJomlMatrix(matrix4f);
        }
        matrix.get(floatValues);
        markDirty();
    }

    @Shadow
    protected abstract void markDirty();

    @Override
    public int getMinecraftType() {
        return type;
    }
}
