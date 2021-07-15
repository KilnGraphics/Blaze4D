package me.hydos.rosella.annotations;

import org.junit.jupiter.api.Tag;

import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

/**
 * These are tests that should be excluded from running during the automatic ci tests.
 * For example tests that are very very slow and or do exhaustive testing.
 */
@Retention(RetentionPolicy.RUNTIME)
@Tag("exclude_frequent_ci")
public @interface ExcludeFrequentCI {
}
