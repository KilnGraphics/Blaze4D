package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.shader.ubo.LowLevelUbo;
import me.hydos.rosella.render.swapchain.SwapChain;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.Window;
import net.minecraft.util.math.Vec3i;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;

import java.nio.ByteBuffer;

import static me.hydos.rosella.render.util.VkUtilsKt.alignas;
import static me.hydos.rosella.render.util.VkUtilsKt.alignof;

public class MinecraftUbo extends LowLevelUbo {

    private int size;

    public Matrix4f projectionMatrix;
    public Matrix4f viewTransformMatrix;

    public MinecraftUbo(@NotNull Device device, @NotNull Memory memory) {
        super(device, memory);
    }

    @Override
    public int getSize() {
        return 260;
    }

    @Override
    public void update(int currentImg, @NotNull SwapChain swapChain, @NotNull Matrix4f view, @NotNull Matrix4f proj, @NotNull Matrix4f modelMatrix) {
        if (getUboFrames().size() == 0) {
            create(swapChain); //TODO: CONCERN. why did i write this
        }

        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer data = stack.mallocPointer(1);
            getMemory().map(getUboFrames().get(currentImg).getAllocation(), false, data);
            ByteBuffer buffer = data.getByteBuffer(0, getSize());
            Window window = MinecraftClient.getInstance().getWindow();

            beginUboWrite();
            putMat4(viewTransformMatrix, buffer); // ModelViewMat
            putMat4(projectionMatrix, buffer); // ProjectionMat
            putVec4f(RenderSystem.getShaderColor(), buffer); // ColorModulator
            putFloat(RenderSystem.getShaderFogStart(), buffer); // FogStart
            putFloat(RenderSystem.getShaderFogEnd(), buffer); // FogEnd
            putVec4f(RenderSystem.getShaderFogColor(), buffer); // FogColor
            putMat4(toJoml(RenderSystem.getTextureMatrix()), buffer); // TextureMat
            putFloat(RenderSystem.getShaderGameTime(), buffer); // GameTime
            putVec2i(window.getFramebufferWidth(), window.getFramebufferHeight(), buffer); // ScreenSize
            putFloat(RenderSystem.getShaderLineWidth(), buffer); // LineWidth

            getMemory().unmap(getUboFrames().get(currentImg).getAllocation());
        }
    }

    protected void putVec2i(int i1, int i2, ByteBuffer buffer) {
        putInt(i1, buffer);
        putInt(i2, buffer);
    }

    protected void putMat4(Matrix4f matrix4f, ByteBuffer buffer) {
        if (size == 0) {
            matrix4f.get(0, buffer);
        } else {
            matrix4f.get(size, buffer);
        }
        size += 16 * Float.BYTES;
    }

    protected void putFloat(float f, ByteBuffer buffer) {
        if (size == 0) {
            buffer.putFloat(f);
        } else {
            buffer.putFloat(size, f);
        }
        size += Float.BYTES;
    }

    protected void putInt(int i, ByteBuffer buffer) {
        if (size == 0) {
            buffer.putInt(i);
        } else {
            buffer.putInt(size, i);
        }
        size += Integer.BYTES;
    }

    protected void putVec4f(float[] vec4, ByteBuffer buffer) {
        putFloat(vec4[0], buffer);
        putFloat(vec4[1], buffer);
        putFloat(vec4[2], buffer);
        putFloat(vec4[3], buffer);
    }

    private void beginUboWrite() {
        size = 0;
    }

    public void setMatrices(Matrix4f projectionMatrix, Matrix4f viewTransformMatrix) {
        this.projectionMatrix = projectionMatrix;
        this.viewTransformMatrix =  viewTransformMatrix;
    }

    public static Matrix4f toJoml(net.minecraft.util.math.Matrix4f mcMatrix) {
        Matrix4f jomlMatrix = new Matrix4f();

        jomlMatrix.m00(mcMatrix.a00);
        jomlMatrix.m01(mcMatrix.a10);
        jomlMatrix.m02(mcMatrix.a20);
        jomlMatrix.m03(mcMatrix.a30);

        jomlMatrix.m10(mcMatrix.a01);
        jomlMatrix.m11(mcMatrix.a11);
        jomlMatrix.m12(mcMatrix.a21);
        jomlMatrix.m13(mcMatrix.a31);

        jomlMatrix.m20(mcMatrix.a02);
        jomlMatrix.m21(mcMatrix.a12);
        jomlMatrix.m22(mcMatrix.a22);
        jomlMatrix.m23(mcMatrix.a32);

        jomlMatrix.m30(mcMatrix.a03);
        jomlMatrix.m31(mcMatrix.a13);
        jomlMatrix.m32(mcMatrix.a23);
        jomlMatrix.m33(mcMatrix.a33);

        return jomlMatrix;
    }

    public static Matrix4f projectionToVulkan(Matrix4f glProjMatrix) {
        Matrix4f vpm = new Matrix4f();

        /*To Convert A OpenGL Projection Matrix to a Vulkan one, we need to do the following multiplication...
        | 1 (m00)   0   0    0   |
        | 0  1 (m11)   0    0   |
        | 0   0   0.5 (m22)  0 |
        | 0   0   0.5 (m23)    1 (m33)|*/

        vpm.m00(1);
        vpm.m11(1);
        vpm.m22(0.5f);
        vpm.m23(0.5f);
        vpm.m33(1);
        glProjMatrix.mul(vpm);
        return glProjMatrix;
    }
}
