package me.hydos.rosella.device.init.features;

import me.hydos.rosella.device.init.DeviceBuildConfigurator;
import me.hydos.rosella.device.init.DeviceBuildInformation;
import me.hydos.rosella.device.init.DeviceBuilder;
import me.hydos.rosella.util.NamedID;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.*;

/**
 * <p>A class that represents some collection of device features or capabilities.</p>
 *
 * <p>Instances of this class can be registered into a {@link me.hydos.rosella.device.init.InitializationRegistry} which will then be
 * used to select and initialize a device.</p>
 *
 * <p>This happens in 2 stages.
 * <ol>
 *     <li>The feature is queried if the device supports the feature.</li>
 *     <li>If support is detected and desired the feature will be called to configure the device.</li>
 * </ol>
 * For these interactions a instance of {@link me.hydos.rosella.device.init.DeviceBuilder.DeviceMeta} is provided which manages
 * information for a single physical device.</p>
 *
 * <p>Since multiple devices may be tested concurrently the createInstance function will be called for each device which
 * should return a object that can keep track of all necessary metadata it may need for one device. The ApplicationFeature
 * class as well as separate created instances may be called concurrently, however created instances individually will
 * never be called concurrently.</p>
 *
 * <p>If the feature wants to return information to the application it can provide a metadata object which will be stored
 * in the created device for the application to access.</p>
 *
 * <p>A feature can access the instances of other features, however it must make sure to declare dependencies as otherwise
 * those features may not have run yet.</p>
 *
 * <p>The default implementation of this class only validates that all dependencies are met and does not create any metadata.</p>
 */
public abstract class ApplicationFeature {

    public final NamedID name;
    public final Set<NamedID> dependencies;

    /**
     * It is recommended to use the standard minecraft syntax for names. (i.e. "mod:name")
     *
     * @param name The name of this feature
     */
    public ApplicationFeature(@NotNull NamedID name) {
        this.name = name;
        this.dependencies = Collections.emptySet();
    }

    /**
     * It is recommended to use the standard minecraft syntax for names. (i.e. "mod:name")
     *
     * @param name The name of this feature
     * @param dependencies A list of dependencies
     */
    public ApplicationFeature(@NotNull NamedID name, @Nullable Collection<NamedID> dependencies) {
        this.name = name;
        if(dependencies != null) {
            this.dependencies = Set.copyOf(dependencies);
        } else {
            this.dependencies = Collections.emptySet();
        }
    }

    /**
     * @return A new instance to process a device
     */
    public abstract Instance createInstance();

    @Override
    public final boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ApplicationFeature that = (ApplicationFeature) o;
        return name.equals(that.name);
    }

    @Override
    public final int hashCode() {
        return name.hashCode();
    }

    /**
     * A class to process one device. A instance will never be reused.
     */
    public abstract class Instance {

        protected boolean canEnable;

        public final NamedID getFeatureName() {
            return name;
        }

        public boolean isSupported() {
            return this.canEnable;
        }

        /**
         * Tests if all dependent features are supported.
         *
         * @param meta The DeviceMeta instance.
         * @return True if all dependant features are supported. False otherwise.
         */
        protected boolean allDependenciesMet(DeviceBuildInformation meta) {
            return dependencies.stream().allMatch(meta::isApplicationFeatureSupported);
        }

        /**
         * Tests if the device supports this feature. All dependant features will have had this function called already,
         * and can be accessed by the DeviceMeta instance provided. This function <b>must</b> set the canEnable boolean.
         *
         * @param meta A DeviceMeta instance used to track information about the build process.
         */
        public abstract void testFeatureSupport(DeviceBuildInformation meta);

        /**
         * Should configure the device to enable this feature.
         *
         * @param meta A DeviceMeta instance used to track information about the build process.
         * @return A object that can be used to return information to the application. Can be null.
         */
        public abstract Object enableFeature(DeviceBuildConfigurator meta);
    }
}
