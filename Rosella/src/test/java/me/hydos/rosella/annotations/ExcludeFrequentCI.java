package me.hydos.rosella.annotations;

import org.junit.jupiter.api.Tag;

/**
 * These are tests that should be excluded from running during the automatic ci tests.
 * For example tests that are very slow.
 */
@Tag("exclude_frequent_ci")
public @interface ExcludeFrequentCI {
}
