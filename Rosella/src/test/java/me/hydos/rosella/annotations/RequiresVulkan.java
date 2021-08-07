package me.hydos.rosella.annotations;

import org.junit.jupiter.api.Tag;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

/**
 * These are tests that require the vulkan runtime to run.
 */
@Retention(RetentionPolicy.RUNTIME)
@Tag("requires_vulkan")
public @interface RequiresVulkan {
}
