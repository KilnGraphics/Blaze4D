package me.hydos.blaze4d.api.shader;

/*      #version 150 -> #version 450
        + #extension GL_ARB_separate_shader_objects : enable

        in vec3 Position; -> layout(location = 0) in vec3 Position;
        in vec2 UV; -> layout(location = 1) in vec2 UV;
        in vec4 Color; -> layout(location = 2) in vec4 Color;

        uniform mat4 ModelViewMat; X
        uniform mat4 ProjMat; X

        + layout(binding = 0) uniform UniformBufferObject {
        +     mat4 ModelViewMat;
        +     mat4 ProjMat;
        + } ubo;

        out vec2 texCoord; -> layout(location = 0) out vec2 texCoord;
        out vec4 vertexColor; -> layout(location = 1) out vec4 vertexColor;

        void main() {
        gl_Position = ProjMat * ModelViewMat * vec4(Position, 1.0); -> gl_Position = ubo.ProjMat * ubo.ModelViewMat * vec4(Position, 1.0);

        texCoord = UV;
        vertexColor = Color;
        }*/

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import net.minecraft.client.gl.GlUniform;

public class OpenGLToVulkanShaderProcessor {

    public static List<String> convertOpenGLToVulkanShader(List<String> source, List<GlUniform> glUniforms) {
        List<String> lines = new ArrayList<>(source.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList());

        int inVariables = 0;
        int outVariables = 0;
        int samplers = 1;

        for (int i = 0; i < lines.size(); i++) {
            String line = lines.get(i);

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
                if(!uniformMatcher.find()){
                    throw new RuntimeException("Unable to parse shader line: " + line);
                }
                String type = uniformMatcher.group(1);
                if (type.equals("sampler2D")) {
                    lines.set(i, line.replace("uniform", "layout(binding = " + samplers++ + ") uniform"));
                } else {
                    lines.remove(i);
                    i--;
                }
            } else if (line.matches("void main\\(\\) \\{")) {
//                List<String> uboNames = List.of(
//                        "ModelViewMat",
//                        "ProjMat",
//                        "ColorModulator",
//                        "FogStart",
//                        "FogEnd",
//                        "FogColor",
//                        "TextureMat",
//                        "GameTime",
//                        "ScreenSize",
//                        "LineWidth",
//                        "ChunkOffset",
//                        "Light0_Direction",
//                        "Light1_Direction"
//                );

                List<String> uboNames = glUniforms.stream().map(GlUniform::getName).toList();

                for (String uboName : uboNames) {
                    for (int j = 0; j < lines.size(); j++) {
                        lines.set(j, lines.get(j).replaceAll(uboName, "ubo." + uboName));
                    }
                }

                List<String> uboImports = glUniforms.stream().map(glUniform -> String.format("%s %s;", getDataTypeName(glUniform.getDataType()), glUniform.getName())).toList();
                StringBuilder uboInsert = new StringBuilder("layout(binding = 0) uniform UniformBufferObject {\n");
                uboImports.forEach(string -> uboInsert.append("\t").append(string).append("\n"));
                uboInsert.append("} ubo;\n\n");

                lines.set(i, uboInsert + line);
            }

        }

        return lines.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList();
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
            default -> throw new IllegalStateException("Unexpected value: " + dataType);
        };
    }

    public static void main(String[] args) {
        String originalShader = """
                #version 150

                #moj_import <light.glsl>

                in vec3 Position;
                in vec4 Color;
                in vec2 UV0;
                in ivec2 UV2;
                in vec3 Normal;

                uniform sampler2D Sampler2;

                uniform mat4 ModelViewMat;
                uniform mat4 ProjMat;
                uniform vec3 ChunkOffset;

                out float vertexDistance;
                out vec4 vertexColor;
                out vec2 texCoord0;
                out vec4 normal;

                void main() {
                    gl_Position = ProjMat * ModelViewMat * vec4(Position + ChunkOffset, 1.0);

                    vertexDistance = length((ModelViewMat * vec4(Position + ChunkOffset, 1.0)).xyz);
                    vertexColor = Color * minecraft_sample_lightmap(Sampler2, UV2);
                    texCoord0 = UV0;
                    normal = ProjMat * ModelViewMat * vec4(Normal, 0.0);
                }
                """;
        System.out.println(String.join("\n", convertOpenGLToVulkanShader(List.of(originalShader), Map.of("ModelViewMat", 10, "ProjMat", 10, "ChunkOffset", 6).entrySet().stream().map(entry -> new GlUniform(entry.getKey(), entry.getValue(), 0, null)).toList())));
    }
}
