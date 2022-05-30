package graphics.kiln.blaze4d.api;

import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.Blaze4DNatives;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;
import jdk.incubator.foreign.ValueLayout;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.APIUtil;

public class Blaze4DCore {

    private final MemoryAddress instance;
    private final long b4dWindow;

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
    }

    public void destroy() {
        Blaze4DNatives.b4dDestroy(this.instance);
        GLFW.glfwDestroyWindow(this.b4dWindow);
    }

    public void start_frame() {
        int[] width = new int[1];
        int[] height = new int[1];
        GLFW.glfwGetWindowSize(b4dWindow, width, height);

        MemoryAddress pass = Blaze4DNatives.b4dStartFrame(this.instance, width[0], height[0]);
        if (pass.equals(MemoryAddress.NULL)) {
            Blaze4D.LOGGER.error("Recieved NULL pass");
        } else{
            Blaze4DNatives.b4dEndFrame(pass);
        }
    }
}
