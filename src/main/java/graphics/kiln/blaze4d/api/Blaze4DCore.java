package graphics.kiln.blaze4d.api;

import graphics.kiln.blaze4d.Blaze4DNatives;
import jdk.incubator.foreign.MemoryAddress;
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
    }

    public void destroy() {
        Blaze4DNatives.b4dDestroy(this.instance);
        GLFW.glfwDestroyWindow(this.b4dWindow);
    }
}
