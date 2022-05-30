package graphics.kiln.blaze4d.api;

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

public class Blaze4DCore {

    private final MemoryAddress instance;
    private final long b4dWindow;

    private MemoryAddress currentPass;

    private MemorySegment meshData;

    public Blaze4DCore() {
        GLFW.glfwDefaultWindowHints();
        GLFW.glfwWindowHint(GLFW.GLFW_CLIENT_API, GLFW.GLFW_NO_API);
        this.b4dWindow = GLFW.glfwCreateWindow(800, 600, "BLAAAAAZEEEE 4DDDDD", 0, 0);

        MemoryAddress surfaceProvider = Blaze4DNatives.b4dCreateGlfwSurfaceProvider(this.b4dWindow,
                MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwGetRequiredInstanceExtensions")),
                MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwCreateWindowSurface"))
        );

        this.instance = Blaze4DNatives.b4dInit(surfaceProvider, true);

        MemorySegment format = MemorySegment.allocateNative(Blaze4DNatives.vertexFormatLayout, ResourceScope.globalScope());
        format.setAtIndex(ValueLayout.JAVA_INT, 0, 3);
        format.setAtIndex(ValueLayout.JAVA_INT, 1, 12);
        format.setAtIndex(ValueLayout.JAVA_INT, 2, 0);
        format.setAtIndex(ValueLayout.JAVA_INT, 3, 106);

        Blaze4DNatives.b4dSetVertexFormats(this.instance, format.address(), 1);

        this.meshData = MemorySegment.allocateNative(Blaze4DNatives.meshDataLayout, ResourceScope.globalScope());
    }

    public void destroy() {
        Blaze4DNatives.b4dDestroy(this.instance);
        GLFW.glfwDestroyWindow(this.b4dWindow);
    }

    public long createStaticMesh(ByteBuffer data, int indexOffset, int vertexSize, int indexCount) {
        assert (indexOffset >= 0);

        MemoryAddress vertexPtr = MemoryAddress.ofLong(MemoryUtil.memAddress(data));
        long vertexLen = indexOffset;

        MemoryAddress indexPtr = vertexPtr.addOffset(indexOffset);
        long indexLen = data.remaining() - indexOffset;

        Blaze4D.LOGGER.error("HUH??. " + vertexPtr + " " + vertexLen + " " + indexPtr + " " + indexLen);

        this.meshData.set(ValueLayout.ADDRESS, 0, vertexPtr);
        this.meshData.set(ValueLayout.JAVA_LONG, 8, vertexLen);
        this.meshData.set(ValueLayout.ADDRESS, 16, indexPtr);
        this.meshData.set(ValueLayout.JAVA_LONG, 24, indexLen);
        this.meshData.set(ValueLayout.JAVA_INT, 32, vertexSize);
        this.meshData.set(ValueLayout.JAVA_INT, 36, indexCount);

        return Blaze4DNatives.b4dCreateStaticMesh(this.instance, this.meshData.address());
    }

    public void destroyStaticMesh(long meshId) {
        Blaze4DNatives.b4dDestroyStaticMesh(this.instance, meshId);
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

    public void endFrame() {
        if (this.currentPass == null) {
            Blaze4D.LOGGER.error("Called endFrame when no current pass exists");
        } else {
            Blaze4DNatives.b4dEndFrame(this.currentPass);
            this.currentPass = null;
        }
    }
}
