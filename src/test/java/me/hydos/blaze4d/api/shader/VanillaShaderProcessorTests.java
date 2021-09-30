package me.hydos.blaze4d.api.shader;

import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import graphics.kiln.rosella.render.shader.ShaderType;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.URISyntaxException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public class VanillaShaderProcessorTests {

    private static void testShader(String name) throws URISyntaxException, IOException {
        String path = "me/hydos/blaze4d/api/shader/";
        List<String> lines = readFileLines(path, name + ".txt");
        ShaderType type = switch (lines.get(0).split(": ")[1]) {
            case "vertex" -> ShaderType.VERTEX_SHADER;
            case "fragment" -> ShaderType.FRAGMENT_SHADER;
            default -> throw new IllegalArgumentException("Unknown shader type: " + lines.get(0).split(": ")[1]);
        };

        Map<String, Integer> uniforms = new LinkedHashMap<>();

        for (String uniform : lines.get(1).split(": ")[1].split("; ")) {
            String[] split = uniform.split(", ");
            uniforms.put(split[0], Integer.parseInt(split[1]));
        }

        Assertions.assertEquals(
                String.join("\n", readFileLines(path, name + ".spriv")),
                String.join("\n", VanillaShaderProcessor.process(
                        readFileLines(path, name + ".glsl"),
                        uniforms,
                        new Object2IntOpenHashMap<>(),
                        0
                ).lines()),
                name + " was not converted properly"
        );
    }

    private static List<String> readFileLines(String jarPath, String file) throws URISyntaxException, IOException {
        return Files.readAllLines(Path.of(VanillaShaderProcessorTests.class.getClassLoader().getResource(jarPath + file).toURI()));
    }

    @Test
    public void testShaders() throws URISyntaxException, IOException {
        testShader("test"); // create more tests when new issues arise in the parser.
        testShader("rendertype_outline");
    }
}
