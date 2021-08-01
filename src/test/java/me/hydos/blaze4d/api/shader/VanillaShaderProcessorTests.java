package me.hydos.blaze4d.api.shader;

import it.unimi.dsi.fastutil.Pair;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectIntImmutablePair;
import me.hydos.rosella.render.shader.ShaderType;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;

public class VanillaShaderProcessorTests {
    @Test
    public void testShaders() throws URISyntaxException, IOException {
        testShader("test"); // create more tests when new issues arise in the parser.
    }

    private static void testShader(String name) throws URISyntaxException, IOException {
        String path = "me/hydos/blaze4d/api/shader/";
        List<String> lines = readFileLines(path, name + ".txt");
        ShaderType type = switch (lines.get(0).split(": ")[1]) {
            case "vertex" -> ShaderType.VERTEX_SHADER;
            case "fragment" -> ShaderType.FRAGMENT_SHADER;
            default -> throw new IllegalArgumentException("Unknown shader type: " + lines.get(0).split(": ")[1]);
        };
        List<Pair<String, Integer>> uniforms = Arrays.stream(lines.get(1).split(": ")[1].split("; ")).map(uniform -> (Pair<String, Integer>) new ObjectIntImmutablePair<>(uniform.split(", ")[0], Integer.parseInt(uniform.split(", ")[1]))).toList();
        Assertions.assertEquals(
                readFileLines(path, name + ".spriv"),
                VanillaShaderProcessor.process(
                        readFileLines(path, name + ".glsl"),
                        uniforms,
                        new Object2IntOpenHashMap<>(),
                        0
                ).key()
        );
    }

    private static String readFile(String jarPath, String file) throws URISyntaxException, IOException {
        return Files.readString(Path.of(VanillaShaderProcessorTests.class.getClassLoader().getResource(jarPath + file).toURI()));
    }

    private static List<String> readFileLines(String jarPath, String file) throws URISyntaxException, IOException {
        return Files.readAllLines(Path.of(VanillaShaderProcessorTests.class.getClassLoader().getResource(jarPath + file).toURI()));
    }
}
