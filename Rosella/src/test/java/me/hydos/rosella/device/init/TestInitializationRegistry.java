package me.hydos.rosella.device.init;

import me.hydos.rosella.device.init.features.ApplicationFeature;
import me.hydos.rosella.device.init.features.SimpleApplicationFeature;
import me.hydos.rosella.util.NamedID;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.*;

public class TestInitializationRegistry {

    @Test
    void testApplicationFeatureSortingIndividual() {
        List<NamedID> names = List.of(new NamedID("testing:test1"), new NamedID("testing:test2"), new NamedID("testing:test3"), new NamedID("testing:test4"));
        List<ApplicationFeature> features = new ArrayList<>();
        names.forEach(name -> features.add(new SimpleApplicationFeature(name, null)));

        InitializationRegistry registry = new InitializationRegistry();
        features.forEach(registry::registerApplicationFeature);

        List<ApplicationFeature> result = registry.getOrderedFeatures();
        assertTrue(features.containsAll(result));
    }

    @Test
    void testApplicationFeatureSortingSingeGroup() {
        List<ApplicationFeature> features = new ArrayList<>();
        features.add(new SimpleApplicationFeature(new NamedID("testing:test1"), List.of(new NamedID("testing:test2"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test2"), List.of(new NamedID("testing:test3"), new NamedID("testing:test4"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test3"), null));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test4"), null));

        Random rand = new Random(479821392);
        Collections.shuffle(features, rand);

        InitializationRegistry registry = new InitializationRegistry();
        features.forEach(registry::registerApplicationFeature);

        List<ApplicationFeature> result = registry.getOrderedFeatures();

        Set<NamedID> previousFeatures = new HashSet<>();
        for(ApplicationFeature feature : result) {
            assertTrue(previousFeatures.containsAll(feature.dependencies), "Failed while testing " + feature.name);
            previousFeatures.add(feature.name);
        }
    }

    @Test
    void testApplicationFeatureSortingMultiGroup() {
        List<ApplicationFeature> features = new ArrayList<>();
        features.add(new SimpleApplicationFeature(new NamedID("testing:test1"), List.of(new NamedID("testing:test2"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test2"), List.of(new NamedID("testing:test3"), new NamedID("testing:test4"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test3"), null));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test4"), null));

        features.add(new SimpleApplicationFeature(new NamedID("testing:test6"), List.of(new NamedID("testing:test7"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test5"), null));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test7"), List.of(new NamedID("testing:test5"))));

        features.add(new SimpleApplicationFeature(new NamedID("testing:test9"), null));

        features.add(new SimpleApplicationFeature(new NamedID("testing:test10"), null));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test11"), List.of(new NamedID("testing:test10"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test12"), List.of(new NamedID("testing:test10"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test13"), List.of(new NamedID("testing:test11"), new NamedID("testing:test12"))));

        Random rand = new Random(58234902);
        Collections.shuffle(features, rand);

        InitializationRegistry registry = new InitializationRegistry();
        features.forEach(registry::registerApplicationFeature);

        List<ApplicationFeature> result = registry.getOrderedFeatures();

        Set<NamedID> previousFeatures = new HashSet<>();
        for(ApplicationFeature feature : result) {
            assertTrue(previousFeatures.containsAll(feature.dependencies), "Failed while testing " + feature.name);
            previousFeatures.add(feature.name);
        }
    }

    @Test
    void testApplicationFeatureSortingCycle() {
        List<ApplicationFeature> features = new ArrayList<>();
        features.add(new SimpleApplicationFeature(new NamedID("testing:test1"), List.of(new NamedID("testing:test2"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test2"), List.of(new NamedID("testing:test3"), new NamedID("testing:test4"))));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test3"), null));
        features.add(new SimpleApplicationFeature(new NamedID("testing:test4"), List.of(new NamedID("testing:test1"))));

        Random rand = new Random(479821392);
        Collections.shuffle(features, rand);

        InitializationRegistry registry = new InitializationRegistry();
        features.forEach(registry::registerApplicationFeature);

        assertThrows(RuntimeException.class, registry::getOrderedFeatures);
    }
}
