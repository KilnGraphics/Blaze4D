package me.hydos.blaze4d.api.shader;

/*      #version 150 -> #version 450
        + #extension GL_ARB_separate_shader_objects : enable

        in vec3 Position; -> layout(location = 0) in vec3 Position;
        in vec2 UV; -> layout(location = 1) in vec2 UV;
        in vec4 Color; -> layout(location = 2) in vec4 Color;

        uniform mat4 ModelViewMat; X
        uniform mat4 ProjMat; X

        + layout(binding = 0) uniform UniformBufferObject {layout(binding = 0) uniform UniformBufferObject {
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
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class OpenGLToVulkanShaderProcessor {
    public static List<String> convertOpenGLToVulkanShader(List<String> source) {
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
                uniformMatcher.find();
                String type = uniformMatcher.group(1);
                if (type.equals("sampler2D")) {
                    lines.set(i, line.replace("uniform", "layout(binding = " + samplers++ + ") uniform"));
                } else {
                    lines.set(i, "");
                }
            } else if (line.matches("void main\\(\\) \\{")) {
                List<String> uboNames = List.of(
                        "ModelViewMat",
                        "ProjMat",
                        "ColorModulator",
                        "FogStart",
                        "FogEnd",
                        "FogColor",
                        "TextureMat",
                        "GameTime",
                        "ScreenSize",
                        "LineWidth"
                );

                for (String uboName : uboNames) {
                    for (int j = 0; j < lines.size(); j++) {
                        lines.set(j, lines.get(j).replaceAll(uboName, "ubo." + uboName));
                    }
                }

                String uboInsert = """
                        layout(binding = 0) uniform UniformBufferObject {
                            mat4 ModelViewMat;
                            mat4 ProjMat;
                            vec4 ColorModulator;
                            float FogStart;
                            float FogEnd;
                            vec4 FogColor;
                            mat4 TextureMat;
                            float GameTime;
                            vec2 ScreenSize;
                            float LineWidth;
                        } ubo;
                                                
                        """;
                lines.set(i, uboInsert + line);
            }

        }

        return lines.stream()
                .flatMap(line -> Arrays.stream(line.split("\n")))
                .toList();
    }

    public static void main(String[] args) {
        String originalShader = """
                #version 150

                #moj_import <fog.glsl>

                uniform sampler2D Sampler0;

                uniform vec4 ColorModulator;
                uniform float FogStart;
                uniform float FogEnd;
                uniform vec4 FogColor;

                in float vertexDistance;
                in vec4 vertexColor;
                in vec4 lightMapColor;
                in vec4 overlayColor;
                in vec2 texCoord0;
                in vec4 normal;

                out vec4 fragColor;

                void main() {
                    vec4 color = texture(Sampler0, texCoord0);
                    if (color.a < 0.1) {
                        discard;
                    }
                    color *= vertexColor * ColorModulator;
                    color.rgb = mix(overlayColor.rgb, color.rgb, overlayColor.a);
                    color *= lightMapColor;
                    fragColor = linear_fog(color, vertexDistance, FogStart, FogEnd, FogColor);
                }
                """;
        System.out.println(String.join("\n", convertOpenGLToVulkanShader(List.of(originalShader))));
    }
}
