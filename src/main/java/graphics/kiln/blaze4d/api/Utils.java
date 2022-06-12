package graphics.kiln.blaze4d.api;

import com.google.common.collect.ImmutableList;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.blaze3d.vertex.VertexFormatElement;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.core.natives.VertexFormatNative;

public class Utils {
    public static boolean convertVertexFormat(VertexFormat src, VertexFormatNative dst) {
        boolean hasPosition = false;
        dst.reset();
        dst.setStride(src.getVertexSize());

        ImmutableList<String> names = src.getElementAttributeNames();
        ImmutableList<VertexFormatElement> elements = src.getElements();

        int currentOffset = 0;
        for (int i = 0; i < names.size(); i++) {
            String name = names.get(i);
            VertexFormatElement element = elements.get(i);

            switch (name) {
                case "Position" -> {
                    if (element.getType() == VertexFormatElement.Type.FLOAT) {
                        hasPosition = true;
                        dst.setPosition(currentOffset, vulkanF32Format(element.getCount()));
                    } else {
                        Blaze4D.LOGGER.warn("Vertex format position type is not float. Skipping!");
                    }
                }
                case "Normal" -> {
                    dst.setNormal(currentOffset, vulkanNormFormat(element.getType(), element.getCount()));
                }
                case "Color" -> {
                    dst.setColor(currentOffset, vulkanNormFormat(element.getType(), element.getCount()));
                }
                case "UV0" -> {
                    dst.setUV0(currentOffset, vulkanNormFormat(element.getType(), element.getCount()));
                }
                case "UV1" -> {
                    dst.setUV1(currentOffset, vulkanNormFormat(element.getType(), element.getCount()));
                }
                case "UV2" -> {
                    dst.setUV2(currentOffset, vulkanNormFormat(element.getType(), element.getCount()));
                }
            }

            currentOffset += element.getByteSize();
        }

        return hasPosition;
    }

    public static int vulkanNormFormat(VertexFormatElement.Type type, int componentCount) {
        switch (type) {
            case FLOAT -> {
                return vulkanF32Format(componentCount);
            }
            case UBYTE -> {
                return vulkanU8NormFormat(componentCount);
            }
            case BYTE -> {
                return vulkanI8NormFormat(componentCount);
            }
            case USHORT -> {
                return vulkanU16NormFormat(componentCount);
            }
            case SHORT -> {
                return vulkanI16NormFormat(componentCount);
            }
            default -> {
                throw new RuntimeException("32 bit values cannot be normalized");
            }
        }
    }

    public static int vulkanF32Format(int componentCount) {
        switch (componentCount) {
            case 1 -> {
                return 100;
            }
            case 2 -> {
                return 103;
            }
            case 3 -> {
                return 106;
            }
            case 4 -> {
                return 109;
            }
            default -> {
                throw new RuntimeException("Invalid component count " + componentCount);
            }
        }
    }

    public static int vulkanU8NormFormat(int componentCount) {
        switch (componentCount) {
            case 1 -> {
                return 9;
            }
            case 2 -> {
                return 16;
            }
            case 3 -> {
                return 23;
            }
            case 4 -> {
                return 37;
            }
            default -> {
                throw new RuntimeException("Invalid component count " + componentCount);
            }
        }
    }

    public static int vulkanI8NormFormat(int componentCount) {
        switch (componentCount) {
            case 1 -> {
                return 10;
            }
            case 2 -> {
                return 17;
            }
            case 3 -> {
                return 24;
            }
            case 4 -> {
                return 38;
            }
            default -> {
                throw new RuntimeException("Invalid component count " + componentCount);
            }
        }
    }

    public static int vulkanU16NormFormat(int componentCount) {
        switch (componentCount) {
            case 1 -> {
                return 70;
            }
            case 2 -> {
                return 77;
            }
            case 3 -> {
                return 84;
            }
            case 4 -> {
                return 91;
            }
            default -> {
                throw new RuntimeException("Invalid component count " + componentCount);
            }
        }
    }

    public static int vulkanI16NormFormat(int componentCount) {
        switch (componentCount) {
            case 1 -> {
                return 71;
            }
            case 2 -> {
                return 78;
            }
            case 3 -> {
                return 85;
            }
            case 4 -> {
                return 92;
            }
            default -> {
                throw new RuntimeException("Invalid component count " + componentCount);
            }
        }
    }
}
