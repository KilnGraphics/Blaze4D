package graphics.kiln.blaze4d.api;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.math.Matrix4f;
import com.mojang.math.Vector3f;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.Blaze4DNatives;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;
import jdk.incubator.foreign.ValueLayout;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.APIUtil;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.util.Arrays;

public class Blaze4DCore {

    private final MemoryAddress instance;
    private final long b4dWindow;

    private MemoryAddress currentPass;

    private final MemorySegment meshData;
    private final MemorySegment devUniformData;

    public Blaze4DCore() {
        GLFW.glfwDefaultWindowHints();
        GLFW.glfwWindowHint(GLFW.GLFW_CLIENT_API, GLFW.GLFW_NO_API);
        this.b4dWindow = GLFW.glfwCreateWindow(800, 600, "BLAAAAAZEEEE 4DDDDD", 0, 0);

        MemoryAddress surfaceProvider = Blaze4DNatives.b4dCreateGlfwSurfaceProvider(this.b4dWindow,
                MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwGetRequiredInstanceExtensions")),
                MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwCreateWindowSurface"))
        );

        this.instance = Blaze4DNatives.b4dInit(surfaceProvider, true);

        this.meshData = MemorySegment.allocateNative(Blaze4DNatives.meshDataLayout, ResourceScope.globalScope());
        this.devUniformData = MemorySegment.allocateNative(144, ResourceScope.globalScope());
    }

    public void destroy() {
        Blaze4DNatives.b4dDestroy(this.instance);
        GLFW.glfwDestroyWindow(this.b4dWindow);
    }

    public long createStaticMesh(ByteBuffer vertexData, ByteBuffer indexData, int vertexSize, int indexCount, int indexType, int primitiveTopology) {
        MemoryAddress vertexPtr = MemoryAddress.ofLong(MemoryUtil.memAddress(vertexData));
        MemoryAddress indexPtr = MemoryAddress.ofLong(MemoryUtil.memAddress(indexData));

        this.meshData.set(ValueLayout.ADDRESS, 0, vertexPtr);
        this.meshData.set(ValueLayout.JAVA_LONG, 8, vertexData.remaining());
        this.meshData.set(ValueLayout.ADDRESS, 16, indexPtr);
        this.meshData.set(ValueLayout.JAVA_LONG, 24, indexData.remaining());
        this.meshData.set(ValueLayout.JAVA_INT, 32, vertexSize);
        this.meshData.set(ValueLayout.JAVA_INT, 36, indexCount);
        this.meshData.set(ValueLayout.JAVA_INT, 40, indexType);
        this.meshData.set(ValueLayout.JAVA_INT, 44, primitiveTopology);

        return Blaze4DNatives.b4dCreateStaticMesh(this.instance, this.meshData.address());
    }

    public void destroyStaticMesh(long meshId) {
        Blaze4DNatives.b4dDestroyStaticMesh(this.instance, meshId);
    }

    public long createShader(int stride, int offset, int format) {
        return Blaze4DNatives.b4dCreateShader(this.instance, stride, offset, format);
    }

    public void destroyShader(long id) {
        Blaze4DNatives.b4dDestroyShader(this.instance, id);
    }

    public void startFrame() {
        if (this.currentPass != null) {
            Blaze4D.LOGGER.warn("Old pass has not been completed yet");
            Blaze4DNatives.b4dEndFrame(this.currentPass);
            this.currentPass = null;
        }

        int[] width = new int[1];
        int[] height = new int[1];
        GLFW.glfwGetWindowSize(b4dWindow, width, height);

        MemoryAddress pass = Blaze4DNatives.b4dStartFrame(this.instance, width[0], height[0]);
        if (!pass.equals(MemoryAddress.NULL)) {
            this.currentPass = pass;
        }
    }

    public void passUpdateDevUniform(long shaderId, Matrix4f proj, Matrix4f modelView, Vector3f chunk_offset) {
        if (this.currentPass == null) {
            Blaze4D.LOGGER.error("Called passSetModelView when no current pass exists");
        } else {
            this.writeMatrix(this.devUniformData, 0, proj);
            this.writeMatrix(this.devUniformData, 64, modelView);
            this.devUniformData.set(ValueLayout.JAVA_FLOAT, 128, chunk_offset.x());
            this.devUniformData.set(ValueLayout.JAVA_FLOAT, 132, chunk_offset.y());
            this.devUniformData.set(ValueLayout.JAVA_FLOAT, 136, chunk_offset.z());

            Blaze4DNatives.b4dPassUpdateDevUniform(this.currentPass, this.devUniformData.address(), shaderId);
        }
    }
    public void passDrawStatic(long meshId, long shaderId) {
        if (this.currentPass == null) {
            Blaze4D.LOGGER.error("Called passDrawStatic when no current pass exists");
        } else {
            Blaze4DNatives.b4dPassDrawStatic(this.currentPass, meshId, shaderId);
        }
    }

    public void passDrawImmediate(ByteBuffer vertexData, ByteBuffer indexData, int vertexSize, int indexCount, int indexType, int primitiveTopology, long shaderId) {
        if (this.currentPass == null) {
            Blaze4D.LOGGER.error("Called passDrawImmediate when no current pass exists");
        } else {
            MemoryAddress vertexPtr = MemoryAddress.ofLong(MemoryUtil.memAddress(vertexData));
            MemoryAddress indexPtr = MemoryAddress.ofLong(MemoryUtil.memAddress(indexData));

            this.meshData.set(ValueLayout.ADDRESS, 0, vertexPtr);
            this.meshData.set(ValueLayout.JAVA_LONG, 8, vertexData.remaining());
            this.meshData.set(ValueLayout.ADDRESS, 16, indexPtr);
            this.meshData.set(ValueLayout.JAVA_LONG, 24, indexData.remaining());
            this.meshData.set(ValueLayout.JAVA_INT, 32, vertexSize);
            this.meshData.set(ValueLayout.JAVA_INT, 36, indexCount);
            this.meshData.set(ValueLayout.JAVA_INT, 40, indexType);
            this.meshData.set(ValueLayout.JAVA_INT, 44, primitiveTopology);

            Blaze4DNatives.b4dPassDrawImmediate(this.currentPass, this.meshData.address(), shaderId);
        }
    }

    public void endFrame() {
        if (this.currentPass == null) {
            Blaze4D.LOGGER.error("Called endFrame when no current pass exists");
        } else {
            Blaze4DNatives.b4dEndFrame(this.currentPass);
            this.currentPass = null;
        }
    }

    private void writeMatrix(MemorySegment segment, long offset, Matrix4f matrix) {
        segment.set(ValueLayout.JAVA_FLOAT, offset, matrix.m00);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 4, matrix.m10);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 8, matrix.m20);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 12, matrix.m30);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 16, matrix.m01);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 20, matrix.m11);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 24, matrix.m21);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 28, matrix.m31);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 32, matrix.m02);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 36, matrix.m12);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 40, matrix.m22);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 44, matrix.m32);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 48, matrix.m03);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 52, matrix.m13);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 56, matrix.m23);
        segment.set(ValueLayout.JAVA_FLOAT, offset + 60, matrix.m33);
    }
}
