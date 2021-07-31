package me.hydos.blaze4d.api.shader;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import it.unimi.dsi.fastutil.Pair;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectIntImmutablePair;
import it.unimi.dsi.fastutil.objects.ObjectIntPair;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;

public class VanillaShaderProcessor {
    public static final Pattern SHADER_IN_ATTRIBUTE = Pattern.compile("in\\s*\\w*\\s*\\w*;");
    public static final Pattern SHADER_OUT_ATTRIBUTE = Pattern.compile("out\\s*\\w*\\s*\\w*;");
    public static final Pattern UNIFORM = Pattern.compile("uniform\\s(\\w*)\\s(\\w*);");
    public static final Pattern METHOD_WITHOUT_PARAMETERS_SIGNATURE = Pattern.compile("\\w*\\s*\\w*\\(\\)\\s*\\{");
    public static final Pattern METHOD_WITH_PARAMETERS_SIGNATURE = Pattern.compile("\\w*\\s*\\w*\\(([\\w\\s,]*)\\)\\s*\\{");
    public static final Pattern VERSION = Pattern.compile("#version\\s*\\d*");

    public static ObjectIntPair<List<String>> process(List<String> source, List<Pair<String, Integer>> glUniforms, Object2IntMap<String> currentSamplerBindings, int initialSamplerBinding) {
        List<String> lines = new ArrayList<>(source.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList());

        int inVariables = 0;
        int outVariables = 0;
        int samplerBinding = initialSamplerBinding;

        int currentCurlyBracket = 0;
        Set<String> uniformStringShouldBeReplaced = new ObjectOpenHashSet<>(glUniforms.size());

        for (int i = 0; i < lines.size(); i++) {
            for (String uboName : uniformStringShouldBeReplaced) {
                lines.set(i, lines.get(i).replaceAll(uboName, "ubo." + uboName));
            }

            String line = lines.get(i)
                    .replace("gl_VertexID", "gl_VertexIndex")
                    .replace("gl_InstanceID", "gl_InstanceIndex");

            lines.set(i, line);

            if (VERSION.matcher(line).matches()) {
                lines.set(i, """
                        #version 450
                        #extension GL_ARB_separate_shader_objects : enable
                        """);
                List<String> uboImports = glUniforms
                        .stream()
                        .map(glUniform -> String.format("%s %s;", getDataTypeName(glUniform.right()), glUniform.left()))
                        .toList();
                StringBuilder uboInsert = new StringBuilder("layout(binding = 0) uniform UniformBufferObject {\n");
                uboImports.forEach(string -> uboInsert.append("\t").append(string).append("\n"));
                uboInsert.append("} ubo;\n\n");
                lines.set(i + 1, uboInsert + lines.get(i + 1));
                i++;
            } else if (SHADER_IN_ATTRIBUTE.matcher(line).matches()) {
                lines.set(i, "layout(location = " + (inVariables++) + ") " + line);
            } else if (SHADER_OUT_ATTRIBUTE.matcher(line).matches()) {
                lines.set(i, "layout(location = " + (outVariables++) + ") " + line);
            } else if (UNIFORM.matcher(line).matches()) {
                Matcher uniformMatcher = UNIFORM.matcher(line);
                if (!uniformMatcher.find()) {
                    throw new RuntimeException("Unable to parse shader line: " + line);
                }
                String type = uniformMatcher.group(1);
                if (type.equals("sampler2D")) {
                    String name = uniformMatcher.group(2);
                    int existingBinding = currentSamplerBindings.getInt(name);
                    if (existingBinding == -1) {
                        currentSamplerBindings.put(name, samplerBinding);
                        lines.set(i, line.replace("uniform", "layout(binding = " + samplerBinding++ + ") uniform"));
                    } else {
                        lines.set(i, line.replace("uniform", "layout(binding = " + existingBinding + ") uniform"));
                    }
                } else {
                    lines.remove(i);
                    i--;
                }
            } else if (METHOD_WITHOUT_PARAMETERS_SIGNATURE.matcher(line).matches()) {
                uniformStringShouldBeReplaced.addAll(glUniforms.stream().map(Pair::left).toList());
                currentCurlyBracket++;
            } else if (METHOD_WITH_PARAMETERS_SIGNATURE.matcher(line).matches()) {
                Matcher matcher = METHOD_WITH_PARAMETERS_SIGNATURE.matcher(line);
                if (!matcher.find()) {
                    throw new RuntimeException("Unable to read parameters from shader line: " + line);
                }
                String methodParameters = matcher.group(1);
                List<String> notUniformNames = Arrays.stream(methodParameters.split(",\\s*")).map(s -> s.split("\\s+")[1]).toList();
                glUniforms.stream().map(Pair::left).filter(s -> !notUniformNames.contains(s)).forEach(uniformStringShouldBeReplaced::add);
                currentCurlyBracket++;
            } else if (line.contains("{")) {
                currentCurlyBracket++;
            } else if (line.contains("}")) {
                currentCurlyBracket--;
                if (currentCurlyBracket == 0) {
                    uniformStringShouldBeReplaced.clear();
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
        System.out.println(String.join("\n", process(List.of(originalShader), Map.of("GameTime", 4, "EndPortalLayers", 0).entrySet().stream().map(uniform -> (Pair<String, Integer>) new ObjectIntImmutablePair<>(uniform.getKey(), uniform.getValue())).toList(), new Object2IntOpenHashMap<>(), 1).key()));
    }
}
