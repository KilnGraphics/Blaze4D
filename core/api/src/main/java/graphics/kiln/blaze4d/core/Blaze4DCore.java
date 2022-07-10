package graphics.kiln.blaze4d.core;

import graphics.kiln.blaze4d.core.natives.Natives;
import graphics.kiln.blaze4d.core.types.B4DFormat;
import graphics.kiln.blaze4d.core.types.B4DImageData;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import graphics.kiln.blaze4d.core.types.B4DVertexFormat;
import jdk.incubator.foreign.MemoryAddress;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;

public class Blaze4DCore implements AutoCloseable {
    public static final Logger LOGGER = LogManager.getLogger("Blaze4DCore", new StringFormatterMessageFactory());

    private final MemoryAddress handle;

    public Blaze4DCore(long glfwWindow) {
        boolean enableValidation = System.getProperty("b4d.enable_validation") != null;

        MemoryAddress surfaceProvider = Natives.b4dCreateGlfwSurfaceProvider(glfwWindow);
        this.handle = Natives.b4dInit(surfaceProvider, enableValidation);
    }

    public void setDebugMode(DebugMode mode) {
        Natives.b4dSetDebugMode(this.handle, mode.raw);
    }

    public long createShader(B4DVertexFormat vertexFormat, long usedUniforms) {
        return Natives.b4dCreateShader(this.handle, vertexFormat.getAddress(), usedUniforms);
    }

    public void destroyShader(long shaderId) {
        Natives.b4dDestroyShader(this.handle, shaderId);
    }

    public GlobalMesh createGlobalMesh(B4DMeshData meshData) {
        return new GlobalMesh(Natives.b4dCreateGlobalMesh(this.handle, meshData.getAddress()));
    }

    public GlobalImage createGlobalImage(int width, int height, B4DFormat format) {
        return new GlobalImage(Natives.b4dCreateGlobalImage(this.handle, width, height, format.getValue()));
    }

    public Frame startFrame(int windowWidth, int windowHeight) {
        MemoryAddress frame = Natives.b4dStartFrame(this.handle, windowWidth, windowHeight);
        if(frame.toRawLongValue() == 0L) {
            return null;
        } else {
            return new Frame(frame);
        }
    }

    @Override
    public void close() throws Exception {
        Natives.b4dDestroy(this.handle);
    }

    public enum DebugMode {
        NONE(0),
        DEPTH(1),
        POSITION(2),
        COLOR(3),
        NORMAL(4),
        UV0(5),
        UV1(6),
        UV2(7),
        TEXTURED0(8),
        TEXTURED1(9),
        TEXTURED2(10);

        final int raw;

        DebugMode(int raw) {
            this.raw = raw;
        }
    }
}
