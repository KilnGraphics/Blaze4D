package graphics.kiln.blaze4d.mixin.shader;

import com.mojang.blaze3d.shaders.AbstractUniform;
import com.mojang.blaze3d.shaders.Shader;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.math.Vector3f;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.core.types.B4DUniform;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import javax.annotation.Nullable;

@Mixin(Uniform.class)
public abstract class GlUniformMixin extends AbstractUniform implements graphics.kiln.blaze4d.api.B4DUniform {

    @Final
    @Shadow
    private String name;

    @Final
    @Shadow
    private int type;

    @Final
    @Shadow
    private Shader parent;

    @Final
    @Nullable
    private B4DUniform b4dUniform;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void init(String string, int i, int j, Shader shader, CallbackInfo ci) {
        B4DUniform b4dUniform = null;
        switch (this.name) {
            case "ModelViewMat" -> {
                if (this.type == Uniform.UT_MAT4) {
                    b4dUniform = B4DUniform.MODEL_VIEW_MATRIX;
                } else {
                    Blaze4D.LOGGER.warn("Uniform ModelViewMat had type not equal to UT_MAT4. ignoring!");
                }
            }
            case "ProjMat" -> {
                if (this.type == Uniform.UT_MAT4) {
                    b4dUniform = B4DUniform.PROJECTION_MATRIX;
                } else {
                    Blaze4D.LOGGER.warn("Uniform ProjMat had type not equal to UT_MAT4. ignoring!");
                }
            }
            case "IViewRotMat" -> {
                if (this.type == Uniform.UT_MAT4) {
                    b4dUniform = B4DUniform.INVERSE_VIEW_ROTATION_MATRIX;
                } else {
                    Blaze4D.LOGGER.warn("Uniform IViewRotMat had type not equal to UT_MAT4. ignoring!");
                }
            }
            case "TextureMat" -> {
                if (this.type == Uniform.UT_MAT4) {
                    b4dUniform = B4DUniform.TEXTURE_MATRIX;
                } else {
                    Blaze4D.LOGGER.warn("Uniform TextureMat had type not equal to UT_MAT4. ignoring!");
                }
            }
            case "ScreenSize" -> {
                if (this.type == Uniform.UT_FLOAT2) {
                    b4dUniform = B4DUniform.SCREEN_SIZE;
                } else {
                    Blaze4D.LOGGER.warn("Uniform ScreenSize had type not equal to UT_FLOAT2. ignoring!");
                }
            }
            case "ColorModulator" -> {
                if (this.type == Uniform.UT_FLOAT4) {
                    b4dUniform = B4DUniform.COLOR_MODULATOR;
                } else {
                    Blaze4D.LOGGER.warn("Uniform ColorModulator had type not equal to UT_FLOAT4. ignoring!");
                }
            }
            case "Light0_Direction" -> {
                if (this.type == Uniform.UT_FLOAT3) {
                    b4dUniform = B4DUniform.LIGHT0_DIRECTION;
                } else {
                    Blaze4D.LOGGER.warn("Uniform Light0_Direction had type not equal to UT_FLOAT3. ignoring!");
                }
            }
            case "Light1_Direction" -> {
                if (this.type == Uniform.UT_FLOAT3) {
                    b4dUniform = B4DUniform.LIGHT1_DIRECTION;
                } else {
                    Blaze4D.LOGGER.warn("Uniform Light1_Direction had type not equal to UT_FLOAT3. ignoring!");
                }
            }
            case "FogStart" -> {
                if (this.type == Uniform.UT_FLOAT1) {
                    b4dUniform = B4DUniform.FOG_START;
                } else {
                    Blaze4D.LOGGER.warn("Uniform FogStart had type not equal to UT_FLOAT1. ignoring!");
                }
            }
            case "FogEnd" -> {
                if (this.type == Uniform.UT_FLOAT1) {
                    b4dUniform = B4DUniform.FOG_END;
                } else {
                    Blaze4D.LOGGER.warn("Uniform FogEnd had type not equal to UT_FLOAT1. ignoring!");
                }
            }
            case "FogColor" -> {
                if (this.type == Uniform.UT_FLOAT4) {
                    b4dUniform = B4DUniform.FOG_COLOR;
                } else {
                    Blaze4D.LOGGER.warn("Uniform FogColor had type not equal to UT_FLOAT4. ignoring!");
                }
            }
            case "FogShape" -> {
                if (this.type == Uniform.UT_INT1) {
                    b4dUniform = B4DUniform.FOG_SHAPE;
                } else {
                    Blaze4D.LOGGER.warn("Uniform FogShape had type not equal to UT_INT1. ignoring!");
                }
            }
            case "LineWidth" -> {
                if (this.type == Uniform.UT_FLOAT1) {
                    b4dUniform = B4DUniform.LINE_WIDTH;
                } else {
                    Blaze4D.LOGGER.warn("Uniform LineWidth had type not equal to UT_FLOAT1. ignoring!");
                }
            }
            case "GameTime" -> {
                if (this.type == Uniform.UT_FLOAT1) {
                    b4dUniform = B4DUniform.GAME_TIME;
                } else {
                    Blaze4D.LOGGER.warn("Uniform GameTime had type not equal to UT_FLOAT1. ignoring!");
                }
            }
            case "ChunkOffset" -> {
                if (this.type == Uniform.UT_FLOAT3) {
                    b4dUniform = B4DUniform.CHUNK_OFFSET;
                } else {
                    Blaze4D.LOGGER.warn("Uniform ChunkOffset had type not equal to UT_FLOAT3. ignoring!");
                }
            }
        }
        this.b4dUniform = b4dUniform;
    }

    public B4DUniform getB4DUniform() {
        return this.b4dUniform;
    }

    @Inject(method = "set(F)V", at = @At("HEAD"))
    private void setVec1f(float x, CallbackInfo ci) {
    }

    @Inject(method = "set(FFF)V", at = @At("HEAD"))
    private void setVec3f(float x, float y, float z, CallbackInfo ci) {
    }

    @Inject(method = "set(Lcom/mojang/math/Vector3f;)V", at = @At("HEAD"))
    private void setVec3f(Vector3f vec, CallbackInfo ci) {
    }

//    @Unique
//    private long writeLocation;
//
//
//    @Final
//    @Shadow
//    private IntBuffer intValues;
//
//    @Final
//    @Shadow
//    private FloatBuffer floatValues;
//
//    @Final
//    @Shadow
//    private String name;
//
//    @Shadow
//    private boolean dirty;
//
//    @Shadow
//    protected abstract void markDirty();
//
//    @Override
//    public void writeLocation(long address) {
//        writeLocation = address;
//        markDirty();
//    }
//
//    @Inject(method = "upload", at = @At("HEAD"), cancellable = true)
//    public void uploadToRosellaBuffer(CallbackInfo ci) {
//        if (writeLocation == MemoryUtil.NULL || !dirty) {
//            return;
//        }
//
//        this.dirty = false;
//        if (this.type <= 3) {
//            MemoryUtil.memCopy(MemoryUtil.memAddress(intValues), writeLocation, (long) (type + 1) * Integer.BYTES);
//        } else if (this.type <= 7) {
//            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), writeLocation, (long) (type - 3) * Float.BYTES);
//        } else if (this.type <= 10) {
//            MemoryUtil.memCopy(MemoryUtil.memAddress(floatValues), writeLocation, (long) Mth.square(type - 6) * Float.BYTES);
//        } else {
//            throw new UnsupportedOperationException("Uniform has unexpected type " + type);
//        }
//        ci.cancel();
//    }
//
//    @Override
//    public void set(Matrix4f matrix4f) {
//        org.joml.Matrix4f matrix;
//        if (this.name.equals("ProjMat")) {
//            matrix = ConversionUtils.mcToJomlProjectionMatrix(matrix4f);
//        } else {
//            matrix = ConversionUtils.mcToJomlMatrix(matrix4f);
//        }
//        matrix.get(0, floatValues);
//        markDirty();
//    }
//
//    @Override
//    public int alignOffset(int currentOffset) {
//        return switch (type) {
//            case 1, 5 -> Mth.roundToward(currentOffset, 8);
//            case 2, 3, 6, 7, 8, 9, 10 -> Mth.roundToward(currentOffset, 16);
//            default -> currentOffset;
//        };
//    }
}
