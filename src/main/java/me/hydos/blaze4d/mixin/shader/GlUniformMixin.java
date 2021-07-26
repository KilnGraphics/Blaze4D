package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.shaders.AbstractUniform;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.math.Matrix4f;
import me.hydos.blaze4d.api.shader.VulkanUniformBuffer;
import me.hydos.blaze4d.api.util.ConversionUtils;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;

@Mixin(Uniform.class)
public abstract class GlUniformMixin extends AbstractUniform implements VulkanUniformBuffer {
    @Unique
    private ByteBuffer writeLocation;

    @Shadow
    private int type;

    @Shadow
    private IntBuffer intValues;

    @Shadow
    private FloatBuffer floatValues;

    @Shadow
    private String name;

    @Shadow
    private boolean dirty;

    @Override
    public void writeLocation(ByteBuffer buffer) {
        writeLocation = buffer;
    }

    @Inject(method = "upload", at = @At("HEAD"), cancellable = true)
    public void uploadToRosellaBuffer(CallbackInfo ci) {
        if (writeLocation == null) {
            return;
        }

        this.dirty = false;
        if (this.type <= 3) {
            MemoryUtil.memCopy(MemoryUtil.memAddress(intValues), MemoryUtil.memAddress(writeLocation), (long) (type + 1) * Integer.BYTES);
        } else if (this.type <= 7) {
            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), MemoryUtil.memAddress(writeLocation), (long) (type - 3) * Float.BYTES);
        } else {
            if (this.type > 10) {
                return;
            }
            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), MemoryUtil.memAddress(writeLocation), (long) Math.pow(type - 6, 2) * Float.BYTES);
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
}
