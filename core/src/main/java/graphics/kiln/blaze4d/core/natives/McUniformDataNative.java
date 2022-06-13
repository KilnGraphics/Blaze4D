package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.*;

import java.lang.invoke.VarHandle;

public class McUniformDataNative {
    public static final MemoryLayout LAYOUT;

    public static final MemoryLayout.PathElement UNIFORM_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_U32_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_F32_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_VEC2F32_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_VEC3F32_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_VEC4F32_PATH;
    public static final MemoryLayout.PathElement PAYLOAD_MAT4F32_PATH;
    
    public static final VarHandle UNIFORM_HANDLE;
    public static final VarHandle PAYLOAD_U32_HANDLE;
    public static final VarHandle PAYLOAD_F32_HANDLE;
    public static final VarHandle PAYLOAD_VEC2F32_HANDLE;
    public static final VarHandle PAYLOAD_VEC3F32_HANDLE;
    public static final VarHandle PAYLOAD_VEC4F32_HANDLE;
    public static final VarHandle PAYLOAD_MAT4F32_HANDLE;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.JAVA_LONG.withName("uniform"),
                MemoryLayout.unionLayout(
                        ValueLayout.JAVA_INT.withName("u32"),
                        ValueLayout.JAVA_FLOAT.withName("f32"),
                        MemoryLayout.sequenceLayout(2, ValueLayout.JAVA_FLOAT).withName("vec2f32"),
                        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_FLOAT).withName("vec3f32"),
                        MemoryLayout.sequenceLayout(4, ValueLayout.JAVA_FLOAT).withName("vec4f32"),
                        MemoryLayout.sequenceLayout(16, ValueLayout.JAVA_FLOAT).withName("mat4f32")
                ).withName("payload")
        );

        UNIFORM_PATH = MemoryLayout.PathElement.groupElement("uniform");
        PAYLOAD_PATH = MemoryLayout.PathElement.groupElement("payload");
        PAYLOAD_U32_PATH = MemoryLayout.PathElement.groupElement("u32");
        PAYLOAD_F32_PATH = MemoryLayout.PathElement.groupElement("f32");
        PAYLOAD_VEC2F32_PATH = MemoryLayout.PathElement.groupElement("vec2f32");
        PAYLOAD_VEC3F32_PATH = MemoryLayout.PathElement.groupElement("vec3f32");
        PAYLOAD_VEC4F32_PATH = MemoryLayout.PathElement.groupElement("vec4f32");
        PAYLOAD_MAT4F32_PATH = MemoryLayout.PathElement.groupElement("mat4f32");

        UNIFORM_HANDLE = LAYOUT.varHandle(UNIFORM_PATH);
        PAYLOAD_U32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_U32_PATH);
        PAYLOAD_F32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_F32_PATH);
        PAYLOAD_VEC2F32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_VEC2F32_PATH, MemoryLayout.PathElement.sequenceElement());
        PAYLOAD_VEC3F32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_VEC3F32_PATH, MemoryLayout.PathElement.sequenceElement());
        PAYLOAD_VEC4F32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_VEC4F32_PATH, MemoryLayout.PathElement.sequenceElement());
        PAYLOAD_MAT4F32_HANDLE = LAYOUT.varHandle(PAYLOAD_PATH, PAYLOAD_MAT4F32_PATH, MemoryLayout.PathElement.sequenceElement());
    }
}
