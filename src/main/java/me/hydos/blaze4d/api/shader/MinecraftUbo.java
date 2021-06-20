package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.shader.ubo.LowLevelUbo;
import me.hydos.rosella.render.swapchain.SwapChain;
import me.hydos.rosella.render.util.memory.Memory;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;

import java.nio.ByteBuffer;

import static me.hydos.rosella.render.util.VkUtilsKt.alignas;
import static me.hydos.rosella.render.util.VkUtilsKt.alignof;

public class MinecraftUbo extends LowLevelUbo {

    public MinecraftUbo(@NotNull Device device, @NotNull Memory memory) {
        super(device, memory);
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
            int mat4Size = 16 * java.lang.Float.BYTES;

            Matrix4f mcViewModelMatrix = toJoml(RenderSystem.getModelViewMatrix());
            Matrix4f mcProjMatrix = projectionToVulkan(toJoml(RenderSystem.getProjectionMatrix()));

            mcViewModelMatrix.get(0, buffer);
            mcProjMatrix.get(alignas(mat4Size, alignof(mcProjMatrix)), buffer);

            getMemory().unmap(getUboFrames().get(currentImg).getAllocation());
        }
    }

    private Matrix4f toJoml(net.minecraft.util.math.Matrix4f mcMatrix) {
        Matrix4f jomlMatrix = new Matrix4f();

        jomlMatrix.m00(mcMatrix.a00);
        jomlMatrix.m01(mcMatrix.a01);
        jomlMatrix.m02(mcMatrix.a02);
        jomlMatrix.m03(mcMatrix.a03);

        jomlMatrix.m10(mcMatrix.a10);
        jomlMatrix.m11(mcMatrix.a11);
        jomlMatrix.m12(mcMatrix.a12);
        jomlMatrix.m13(mcMatrix.a13);

        jomlMatrix.m20(mcMatrix.a20);
        jomlMatrix.m21(mcMatrix.a21);
        jomlMatrix.m22(mcMatrix.a22);
        jomlMatrix.m23(mcMatrix.a23);

        jomlMatrix.m30(mcMatrix.a30);
        jomlMatrix.m31(mcMatrix.a31);
        jomlMatrix.m32(mcMatrix.a32);
        jomlMatrix.m33(mcMatrix.a33);

        return jomlMatrix;
    }

    private Matrix4f projectionToVulkan(Matrix4f glProjMatrix) {
        Matrix4f vpm = new Matrix4f().set(glProjMatrix);

        /*To Convert A OpenGL Projection Matrix to a Vulkan one, we need to do the following multiplication...
        | 1   0   0    0   |
        | 0  -1   0    0   |
        | 0   0   0.5  0.5 |
        | 0   0   0    1   |*/

        vpm.m00(vpm.m00() * 1);
        vpm.m11(vpm.m11() * -1);
        vpm.m22(vpm.m22() * 0.5f);
        vpm.m23(vpm.m23() * 0.5f);
        vpm.m33(vpm.m33() * 1);

        return vpm;
    }
}
