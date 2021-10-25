package graphics.kiln.blaze4d.util;

import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;

import java.util.*;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class VanillaShaderProcessor {

    private static final Pattern SHADER_IN_ATTRIBUTE = Pattern.compile("in\\s*\\w*\\s*\\w*;");
    private static final Pattern SHADER_OUT_ATTRIBUTE = Pattern.compile("out\\s*\\w*\\s*\\w*;");
    private static final Pattern UNIFORM = Pattern.compile("uniform\\s(\\w*)\\s(\\w*);");
    private static final Pattern METHOD_WITHOUT_PARAMETERS_SIGNATURE = Pattern.compile("\\w*\\s*\\w*\\(\\)\\s*\\{");
    private static final Pattern METHOD_WITH_PARAMETERS_SIGNATURE = Pattern.compile("\\w*\\s*\\w*\\(([\\w\\s,]*)\\)\\s*\\{");
    private static final Pattern VERSION = Pattern.compile("#version\\s*\\d*");
    private static final Pattern TOKEN_PATTERN = Pattern.compile("([\\w_.]+)");

    public static ConversionData process(List<String> source, Map<String, Integer> glUniforms, Object2IntMap<String> currentSamplerBindings, int initialSamplerBinding) {
        List<String> lines = new ArrayList<>(source.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList());

        int inVariables = 0;
        int outVariables = 0;
        int samplerBinding = initialSamplerBinding;

        int currentCurlyBracket = 0;
        Set<String> uniformStringShouldBeReplaced = new ObjectOpenHashSet<>(glUniforms.size());

        for (int i = 0; i < lines.size(); i++) {

            String line = lines.get(i)
                    .replace("gl_VertexID", "gl_VertexIndex")
                    .replace("gl_InstanceID", "gl_InstanceIndex");

            for (String uboName : uniformStringShouldBeReplaced) {
                Matcher wordMatcher = TOKEN_PATTERN.matcher(line);
                while (wordMatcher.find()) {
                    if (wordMatcher.group(1).startsWith(uboName)) {
                        line = line.substring(0, wordMatcher.start(1)) + "ubo." + line.substring(wordMatcher.start(1));
                        wordMatcher = TOKEN_PATTERN.matcher(line);
                    }
                }
            }

            lines.set(i, line);

            if (VERSION.matcher(line).matches()) {
                lines.set(i, """
                        #version 450
                        #extension GL_ARB_separate_shader_objects : enable
                        """);
                List<String> uboImports = glUniforms.entrySet().stream()
                        .map(glUniform -> String.format("%s %s;", getDataTypeName(glUniform.getValue()), glUniform.getKey()))
                        .toList();
                StringBuilder uboInsert = new StringBuilder("layout(binding = 0) uniform UniformBufferObject {\n");
                uboImports.forEach(string -> uboInsert.append("    ").append(string).append("\n"));
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
                uniformStringShouldBeReplaced.addAll(glUniforms.keySet());
                currentCurlyBracket++;
            } else if (METHOD_WITH_PARAMETERS_SIGNATURE.matcher(line).matches()) {
                Matcher matcher = METHOD_WITH_PARAMETERS_SIGNATURE.matcher(line);
                if (!matcher.find()) {
                    throw new RuntimeException("Unable to read parameters from shader line: " + line);
                }
                String methodParameters = matcher.group(1);
                List<String> notUniformNames = Arrays.stream(methodParameters.split(",\\s*")).map(s -> s.split("\\s+")[1]).toList();
                glUniforms.keySet().stream().filter(s -> !notUniformNames.contains(s)).forEach(uniformStringShouldBeReplaced::add);
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

        return new ConversionData(
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

    public record ConversionData(List<String> lines, int samplerBinding) {
    }
}
