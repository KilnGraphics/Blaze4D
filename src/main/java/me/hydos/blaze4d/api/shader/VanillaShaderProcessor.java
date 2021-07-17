package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.shaders.Uniform;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectIntImmutablePair;
import it.unimi.dsi.fastutil.objects.ObjectIntPair;
import me.hydos.blaze4d.api.GlobalRenderSystem;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class VanillaShaderProcessor {

    public static ObjectIntPair<List<String>> process(List<String> source, List<Uniform> glUniforms, Object2IntMap<String> currentSamplerBindings, int initialSamplerBinding) {
        List<String> lines = new ArrayList<>(source.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList());

        int inVariables = 0;
        int outVariables = 0;
        int samplerBinding = initialSamplerBinding;

        for (int i = 0; i < lines.size(); i++) {
            String line = lines.get(i)
                    .replace("gl_VertexID", "gl_VertexIndex")
                    .replace("gl_InstanceID", "gl_InstanceIndex");
            lines.set(i, line);

            if (line.matches("#version \\d*")) {
                lines.set(i, """
                        #version 450
                        #extension GL_ARB_separate_shader_objects : enable
                        """);
            } else if (line.matches("in \\w* \\w*;")) {
                lines.set(i, "layout(location = " + (inVariables++) + ") " + line);
            } else if (line.matches("out \\w* \\w*;")) {
                lines.set(i, "layout(location = " + (outVariables++) + ") " + line);
            } else if (line.matches("uniform .*")) {
                Matcher uniformMatcher = Pattern.compile("uniform\\s(\\w*)\\s(\\w*);").matcher(line);
                if (!uniformMatcher.find()) {
                    throw new RuntimeException("Unable to parse shader line: " + line);
                }
                String type = uniformMatcher.group(1);
                if (type.equals("sampler2D")) {
                    String name = uniformMatcher.group(2);
                    int existingBinding = currentSamplerBindings.getInt(name);
                    if (existingBinding == GlobalRenderSystem.SAMPLER_NOT_BOUND) {
                        currentSamplerBindings.put(name, samplerBinding);
                        lines.set(i, line.replace("uniform", "layout(binding = " + samplerBinding++ + ") uniform"));
                    } else {
                        lines.set(i, line.replace("uniform", "layout(binding = " + existingBinding + ") uniform"));
                    }
                } else {
                    lines.remove(i);
                    i--;
                }
            } else if (line.matches("void main\\(\\) \\{")) {

                List<String> uboNames = glUniforms.stream().map(Uniform::getName).toList();

                for (String uboName : uboNames) {
                    for (int j = i; j < lines.size(); j++) {
                        lines.set(j, lines.get(j).replaceAll(uboName, "ubo." + uboName));
                    }
                }

                List<String> uboImports = glUniforms.stream().map(glUniform -> String.format("%s %s;", getDataTypeName(glUniform.getType()), glUniform.getName())).toList();
                StringBuilder uboInsert = new StringBuilder("layout(binding = 0) uniform UniformBufferObject {\n");
                uboImports.forEach(string -> uboInsert.append("\t").append(string).append("\n"));
                uboInsert.append("} ubo;\n\n");

                lines.set(2, uboInsert + lines.get(2));
            } else if (line.contains("        0.0,")){ // ugly hack for end portals
                System.out.println(line);
                for (Uniform glUniform : glUniforms) {
                    if(line.contains(glUniform.getName())){
                        lines.set(i, line.replace(glUniform.getName(), "ubo." + glUniform.getName()));
                    }
                }
            }

        }

        return new ObjectIntImmutablePair<>(
                lines.stream().flatMap(line -> Arrays.stream(line.split("\n"))).toList(),
                samplerBinding
        );
    }

    private static String getDataTypeName(int dataType) {
        return switch (dataType) {
            case 0 -> "int";
            case 1 -> "ivec2";
            case 2 -> "ivec3";
            case 3 -> "ivec4";
            case 4 -> "float";
            case 5 -> "vec2";
            case 6 -> "vec3";
            case 7 -> "vec4";
            case 10 -> "mat4";
            default -> throw new IllegalStateException("Unexpected Data Type: " + dataType);
        };
    }

    public static void main(String[] args) {
        String originalShader = """
                #version 150

                #moj_import <matrix.glsl>

                uniform sampler2D Sampler0;
                uniform sampler2D Sampler1;

                uniform float GameTime;
                uniform int EndPortalLayers;

                in vec4 texProj0;

                const vec3[] COLORS = vec3[](
                    vec3(0.022087, 0.098399, 0.110818),
                    vec3(0.011892, 0.095924, 0.089485),
                    vec3(0.027636, 0.101689, 0.100326),
                    vec3(0.046564, 0.109883, 0.114838),
                    vec3(0.064901, 0.117696, 0.097189),
                    vec3(0.063761, 0.086895, 0.123646),
                    vec3(0.084817, 0.111994, 0.166380),
                    vec3(0.097489, 0.154120, 0.091064),
                    vec3(0.106152, 0.131144, 0.195191),
                    vec3(0.097721, 0.110188, 0.187229),
                    vec3(0.133516, 0.138278, 0.148582),
                    vec3(0.070006, 0.243332, 0.235792),
                    vec3(0.196766, 0.142899, 0.214696),
                    vec3(0.047281, 0.315338, 0.321970),
                    vec3(0.204675, 0.390010, 0.302066),
                    vec3(0.080955, 0.314821, 0.661491)
                );

                const mat4 SCALE_TRANSLATE = mat4(
                    0.5, 0.0, 0.0, 0.25,
                    0.0, 0.5, 0.0, 0.25,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0
                );

                mat4 end_portal_layer(float layer) {
                    mat4 translate = mat4(
                        1.0, 0.0, 0.0, 17.0 / layer,
                        0.0, 1.0, 0.0, (2.0 + layer / 1.5) * (GameTime * 1.5),
                        0.0, 0.0, 1.0, 0.0,
                        0.0, 0.0, 0.0, 1.0
                    );

                    mat2 rotate = mat2_rotate_z(radians((layer * layer * 4321.0 + layer * 9.0) * 2.0));

                    mat2 scale = mat2((4.5 - layer / 4.0) * 2.0);

                    return mat4(scale * rotate) * translate * SCALE_TRANSLATE;
                }

                out vec4 fragColor;

                void main() {
                    vec3 color = textureProj(Sampler0, texProj0).rgb * COLORS[0];
                    for (int i = 0; i < EndPortalLayers; i++) {
                        color += textureProj(Sampler1, texProj0 * end_portal_layer(float(i + 1))).rgb * COLORS[i];
                    }
                    fragColor = vec4(color, 1.0);
                }
                """;
        System.out.println(String.join("\n", process(List.of(originalShader), Map.of("GameTime", 4, "EndPortalLayers", 0).entrySet().stream().map(entry -> new Uniform(entry.getKey(), entry.getValue(), 0, null)).toList(), new Object2IntOpenHashMap<>(), 1).key()));
    }
}
