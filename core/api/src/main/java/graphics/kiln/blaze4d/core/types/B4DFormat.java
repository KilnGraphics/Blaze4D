package graphics.kiln.blaze4d.core.types;

public enum B4DFormat {
    UNDEFINED(0),
    R8_UNORM(9),
    R8_SNORM(10),
    R8_USCALED(11),
    R8_SSCALED(12),
    R8_UINT(13),
    R8_SINT(14),
    R8_SRGB(15),
    R8G8_UNORM(16),
    R8G8_SNORM(17),
    R8G8_USCALED(18),
    R8G8_SSCALED(19),
    R8G8_UINT(20),
    R8G8_SINT(21),
    R8G8_SRGB(22),
    R8G8B8_UNORM(23),
    R8G8B8_SNORM(24),
    R8G8B8_USCALED(25),
    R8G8B8_SSCALED(26),
    R8G8B8_UINT(27),
    R8G8B8_SINT(28),
    R8G8B8_SRGB(29),
    R8G8B8A8_UNORM(37),
    R8G8B8A8_SNORM(38),
    R8G8B8A8_USCALED(39),
    R8G8B8A8_SSCALED(40),
    R8G8B8A8_UINT(41),
    R8G8B8A8_SINT(42),
    R8G8B8A8_SRGB(43),
    R16_UNORM(70),
    R16_SNORM(71),
    R16_USCALED(72),
    R16_SSCALED(73),
    R16_UINT(74),
    R16_SINT(75),
    R16_SFLOAT(76),
    R16G16_UNORM(77),
    R16G16_SNORM(78),
    R16G16_USCALED(79),
    R16G16_SSCALED(80),
    R16G16_UINT(81),
    R16G16_SINT(82),
    R16G16_SFLOAT(83),
    R16G16B16_UNORM(84),
    R16G16B16_SNORM(85),
    R16G16B16_USCALED(86),
    R16G16B16_SSCALED(87),
    R16G16B16_UINT(88),
    R16G16B16_SINT(89),
    R16G16B16_SFLOAT(90),
    R16G16B16A16_UNORM(91),
    R16G16B16A16_SNORM(92),
    R16G16B16A16_USCALED(93),
    R16G16B16A16_SSCALED(94),
    R16G16B16A16_UINT(95),
    R16G16B16A16_SINT(96),
    R16G16B16A16_SFLOAT(97),
    R32_UINT(98),
    R32_SINT(99),
    R32_SFLOAT(100),
    R32G32_UINT(101),
    R32G32_SINT(102),
    R32G32_SFLOAT(103),
    R32G32B32_UINT(104),
    R32G32B32_SINT(105),
    R32G32B32_SFLOAT(106),
    R32G32B32A32_UINT(107),
    R32G32B32A32_SINT(108),
    R32G32B32A32_SFLOAT(109);

    private final int value;

    B4DFormat(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static B4DFormat fromRaw(int value) {
        switch (value) {
            case 0 -> {
                return B4DFormat.UNDEFINED;
            }
            case 9 -> {
                return B4DFormat.R8_UNORM;
            }
            case 10 -> {
                return B4DFormat.R8_SNORM;
            }
            case 11 -> {
                return B4DFormat.R8_USCALED;
            }
            case 12 -> {
                return B4DFormat.R8_SSCALED;
            }
            case 13 -> {
                return B4DFormat.R8_UINT;
            }
            case 14 -> {
                return B4DFormat.R8_SINT;
            }
            case 15 -> {
                return B4DFormat.R8_SRGB;
            }
            case 16 -> {
                return B4DFormat.R8G8_UNORM;
            }
            case 17 -> {
                return B4DFormat.R8G8_SNORM;
            }
            case 18 -> {
                return B4DFormat.R8G8_USCALED;
            }
            case 19 -> {
                return B4DFormat.R8G8_SSCALED;
            }
            case 20 -> {
                return B4DFormat.R8G8_UINT;
            }
            case 21 -> {
                return B4DFormat.R8G8_SINT;
            }
            case 22 -> {
                return B4DFormat.R8G8_SRGB;
            }
            case 23 -> {
                return B4DFormat.R8G8B8_UNORM;
            }
            case 24 -> {
                return B4DFormat.R8G8B8_SNORM;
            }
            case 25 -> {
                return B4DFormat.R8G8B8_USCALED;
            }
            case 26 -> {
                return B4DFormat.R8G8B8_SSCALED;
            }
            case 27 -> {
                return B4DFormat.R8G8B8_UINT;
            }
            case 28 -> {
                return B4DFormat.R8G8B8_SINT;
            }
            case 29 -> {
                return B4DFormat.R8G8B8_SRGB;
            }
            case 37 -> {
                return B4DFormat.R8G8B8A8_UNORM;
            }
            case 38 -> {
                return B4DFormat.R8G8B8A8_SNORM;
            }
            case 39 -> {
                return B4DFormat.R8G8B8A8_USCALED;
            }
            case 40 -> {
                return B4DFormat.R8G8B8A8_SSCALED;
            }
            case 41 -> {
                return B4DFormat.R8G8B8A8_UINT;
            }
            case 42 -> {
                return B4DFormat.R8G8B8A8_SINT;
            }
            case 43 -> {
                return B4DFormat.R8G8B8A8_SRGB;
            }
            case 70 -> {
                return B4DFormat.R16_UNORM;
            }
            case 71 -> {
                return B4DFormat.R16_SNORM;
            }
            case 72 -> {
                return B4DFormat.R16_USCALED;
            }
            case 73 -> {
                return B4DFormat.R16_SSCALED;
            }
            case 74 -> {
                return B4DFormat.R16_UINT;
            }
            case 75 -> {
                return B4DFormat.R16_SINT;
            }
            case 76 -> {
                return B4DFormat.R16_SFLOAT;
            }
            case 77 -> {
                return B4DFormat.R16G16_UNORM;
            }
            case 78 -> {
                return B4DFormat.R16G16_SNORM;
            }
            case 79 -> {
                return B4DFormat.R16G16_USCALED;
            }
            case 80 -> {
                return B4DFormat.R16G16_SSCALED;
            }
            case 81 -> {
                return B4DFormat.R16G16_UINT;
            }
            case 82 -> {
                return B4DFormat.R16G16_SINT;
            }
            case 83 -> {
                return B4DFormat.R16G16_SFLOAT;
            }
            case 84 -> {
                return B4DFormat.R16G16B16_UNORM;
            }
            case 85 -> {
                return B4DFormat.R16G16B16_SNORM;
            }
            case 86 -> {
                return B4DFormat.R16G16B16_USCALED;
            }
            case 87 -> {
                return B4DFormat.R16G16B16_SSCALED;
            }
            case 88 -> {
                return B4DFormat.R16G16B16_UINT;
            }
            case 89 -> {
                return B4DFormat.R16G16B16_SINT;
            }
            case 90 -> {
                return B4DFormat.R16G16B16_SFLOAT;
            }
            case 91 -> {
                return B4DFormat.R16G16B16A16_UNORM;
            }
            case 92 -> {
                return B4DFormat.R16G16B16A16_SNORM;
            }
            case 93 -> {
                return B4DFormat.R16G16B16A16_USCALED;
            }
            case 94 -> {
                return B4DFormat.R16G16B16A16_SSCALED;
            }
            case 95 -> {
                return B4DFormat.R16G16B16A16_UINT;
            }
            case 96 -> {
                return B4DFormat.R16G16B16A16_SINT;
            }
            case 97 -> {
                return B4DFormat.R16G16B16A16_SFLOAT;
            }
            case 98 -> {
                return B4DFormat.R32_UINT;
            }
            case 99 -> {
                return B4DFormat.R32_SINT;
            }
            case 100 -> {
                return B4DFormat.R32_SFLOAT;
            }
            case 101 -> {
                return B4DFormat.R32G32_UINT;
            }
            case 102 -> {
                return B4DFormat.R32G32_SINT;
            }
            case 103 -> {
                return B4DFormat.R32G32_SFLOAT;
            }
            case 104 -> {
                return B4DFormat.R32G32B32_UINT;
            }
            case 105 -> {
                return B4DFormat.R32G32B32_SINT;
            }
            case 106 -> {
                return B4DFormat.R32G32B32_SFLOAT;
            }
            case 107 -> {
                return B4DFormat.R32G32B32A32_UINT;
            }
            case 108 -> {
                return B4DFormat.R32G32B32A32_SINT;
            }
            case 109 -> {
                return B4DFormat.R32G32B32A32_SFLOAT;
            }
            default ->
                throw new RuntimeException("Invalid format value " + value);
        }
    }
}
