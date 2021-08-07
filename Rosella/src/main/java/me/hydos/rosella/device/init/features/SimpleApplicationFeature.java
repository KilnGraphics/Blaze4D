package me.hydos.rosella.device.init.features;

import me.hydos.rosella.device.init.DeviceBuildConfigurator;
import me.hydos.rosella.device.init.DeviceBuildInformation;
import me.hydos.rosella.util.NamedID;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.Collection;
import java.util.function.Function;

/**
 * A class to create simple features whose test and enable functions do not need to keep any data or for dependency only
 * features.
 *
 * If a test function is provided the feature is only enabled if all dependencies are met and the test function returns true.
 * If no test function is provided the feature will be enabled if all dependencies are met.
 */
public class SimpleApplicationFeature extends ApplicationFeature {

    protected final Function<DeviceBuildInformation, Boolean> testFunc;
    protected final Function<DeviceBuildConfigurator, Void> enableFunc;

    public SimpleApplicationFeature(@NotNull NamedID name, @NotNull Collection<NamedID> dependencies) {
        this(name, dependencies, null, null);
    }

    public SimpleApplicationFeature(@NotNull NamedID name, @Nullable Function<DeviceBuildInformation, Boolean> testFunc, @Nullable Function<DeviceBuildConfigurator, Void> enableFunc) {
        this(name, null, testFunc, enableFunc);
    }

    public SimpleApplicationFeature(@NotNull NamedID name, @Nullable Collection<NamedID> dependencies, @Nullable Function<DeviceBuildInformation, Boolean> testFunc, @Nullable Function<DeviceBuildConfigurator, Void> enableFunc) {
        super(name, dependencies);
        this.testFunc = testFunc;
        this.enableFunc = enableFunc;
    }

    @Override
    public SimpleInstance createInstance() {
        return new SimpleInstance();
    }

    protected class SimpleInstance extends ApplicationFeature.Instance {
        @Override
        public void testFeatureSupport(DeviceBuildInformation meta) {
            this.canEnable = false;
            if(allDependenciesMet(meta)) {
                if(testFunc != null) {
                    this.canEnable = testFunc.apply(meta);
                } else {
                    this.canEnable = true;
                }
            }
        }

        @Override
        public Object enableFeature(DeviceBuildConfigurator meta) {
            if(enableFunc != null) {
                enableFunc.apply(meta);
            }
            return null;
        }
    }
}
