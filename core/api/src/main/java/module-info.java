module graphics.kiln.blaze4d.core {
    requires jdk.incubator.foreign;

    requires com.google.gson;
    requires org.apache.logging.log4j;
    requires org.lwjgl.glfw;
    requires org.apache.commons.lang3;

    exports graphics.kiln.blaze4d.core;
    exports graphics.kiln.blaze4d.core.types;
}