module graphics.kiln.blaze4d.core {
    requires jdk.incubator.foreign;
    requires org.apache.logging.log4j;
    requires org.lwjgl.glfw;

    exports graphics.kiln.blaze4d.core;
    exports graphics.kiln.blaze4d.core.types;
    exports graphics.kiln.blaze4d.core.natives;
}