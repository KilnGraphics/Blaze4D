use std::fmt::{Debug, Formatter};

#[derive(Eq, Copy, Clone, Debug)]
pub struct CompatibilityClass {
    name: &'static str,
}

macro_rules! define_compatibility_class {
    ($name: ident) => {
        pub const $name: CompatibilityClass = CompatibilityClass::new(stringify!($name));
    }
}

impl CompatibilityClass {
    pub const fn new(name: &'static str) -> Self {
        CompatibilityClass { name }
    }

    pub const fn get_name(&self) -> &'static str {
        self.name
    }

    define_compatibility_class!(BIT8);
    define_compatibility_class!(BIT16);
    define_compatibility_class!(BIT24);
    define_compatibility_class!(BIT32);
    define_compatibility_class!(BIT32_G8B8G8R8);
    define_compatibility_class!(BIT32_B8G8R8G8);
    define_compatibility_class!(BIT48);
    define_compatibility_class!(BIT64);
    define_compatibility_class!(BIT64_R10G10B10A10);
    define_compatibility_class!(BIT64_G10B10G10R10);
    define_compatibility_class!(BIT64_B10G10R10G10);
    define_compatibility_class!(BIT64_R12G12B12A12);
    define_compatibility_class!(BIT64_G12B12G12R12);
    define_compatibility_class!(BIT64_B12G12R12G12);
    define_compatibility_class!(BIT64_G16B16G16R16);
    define_compatibility_class!(BIT64_B16G16R16G16);
    define_compatibility_class!(BIT96);
    define_compatibility_class!(BIT128);
    define_compatibility_class!(BIT192);
    define_compatibility_class!(BIT256);
    define_compatibility_class!(BC1_RGB);
    define_compatibility_class!(BC1_RGBA);
    define_compatibility_class!(BC2);
    define_compatibility_class!(BC3);
    define_compatibility_class!(BC4);
    define_compatibility_class!(BC5);
    define_compatibility_class!(BC6H);
    define_compatibility_class!(BC7);
    define_compatibility_class!(ETC2_RGB);
    define_compatibility_class!(ETC2_RGBA);
    define_compatibility_class!(ETC2_EAC_RGBA);
    define_compatibility_class!(EAC_R);
    define_compatibility_class!(EAC_RG);
    define_compatibility_class!(ASTC_4X4);
    define_compatibility_class!(ASTC_5X4);
    define_compatibility_class!(ASTC_5X5);
    define_compatibility_class!(ASTC_6X5);
    define_compatibility_class!(ASTC_6X6);
    define_compatibility_class!(ASTC_8X5);
    define_compatibility_class!(ASTC_8X6);
    define_compatibility_class!(ASTC_8X8);
    define_compatibility_class!(ASTC_10X5);
    define_compatibility_class!(ASTC_10X6);
    define_compatibility_class!(ASTC_10X8);
    define_compatibility_class!(ASTC_10X10);
    define_compatibility_class!(ASTC_12X10);
    define_compatibility_class!(ASTC_12X12);
    define_compatibility_class!(D16);
    define_compatibility_class!(D24);
    define_compatibility_class!(D32);
    define_compatibility_class!(S8);
    define_compatibility_class!(D16S8);
    define_compatibility_class!(D24S8);
    define_compatibility_class!(D32S8);
    define_compatibility_class!(PLANE3_8BIT_420);
    define_compatibility_class!(PLANE2_8BIT_420);
    define_compatibility_class!(PLANE3_8BIT_422);
    define_compatibility_class!(PLANE2_8BIT_422);
    define_compatibility_class!(PLANE3_8BIT_444);
    define_compatibility_class!(PLANE3_10BIT_420);
    define_compatibility_class!(PLANE2_10BIT_420);
    define_compatibility_class!(PLANE3_10BIT_422);
    define_compatibility_class!(PLANE2_10BIT_422);
    define_compatibility_class!(PLANE3_10BIT_444);
    define_compatibility_class!(PLANE3_12BIT_420);
    define_compatibility_class!(PLANE2_12BIT_420);
    define_compatibility_class!(PLANE3_12BIT_422);
    define_compatibility_class!(PLANE2_12BIT_422);
    define_compatibility_class!(PLANE3_12BIT_444);
    define_compatibility_class!(PLANE3_16BIT_420);
    define_compatibility_class!(PLANE2_16BIT_420);
    define_compatibility_class!(PLANE3_16BIT_422);
    define_compatibility_class!(PLANE2_16BIT_422);
    define_compatibility_class!(PLANE3_16BIT_444);
}

impl PartialEq for CompatibilityClass {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.name, other.name)
    }
}

#[derive(Copy, Clone, Eq)]
pub struct ImageFormat {
    format: ash::vk::Format,
    compatibility_class: CompatibilityClass,
}

macro_rules! define_image_format {
    ($name:ident, $compatibility_class:expr, $channel_count:expr) => {
        pub const $name : ImageFormat = ImageFormat::new(ash::vk::Format::$name, $compatibility_class, $channel_count);
    }
}

impl ImageFormat {
    pub const fn new(format: ash::vk::Format, compatibility_class: CompatibilityClass, _channel_count: u32) -> Self {
        ImageFormat { format, compatibility_class }
    }

    pub const fn get_format(&self) -> ash::vk::Format {
        self.format
    }

    pub const fn get_compatibility_class(&self) -> CompatibilityClass {
        self.compatibility_class
    }

    pub fn is_compatible_with(&self, other: &ImageFormat) -> bool {
        self.compatibility_class == other.compatibility_class
    }

    define_image_format!(R4G4_UNORM_PACK8, CompatibilityClass::BIT8, 2);
    define_image_format!(R4G4B4A4_UNORM_PACK16, CompatibilityClass::BIT16, 4);
    define_image_format!(B4G4R4A4_UNORM_PACK16, CompatibilityClass::BIT16, 4);
    define_image_format!(R5G6B5_UNORM_PACK16, CompatibilityClass::BIT16, 3);
    define_image_format!(B5G6R5_UNORM_PACK16, CompatibilityClass::BIT16, 3);
    define_image_format!(R5G5B5A1_UNORM_PACK16, CompatibilityClass::BIT16, 4);
    define_image_format!(B5G5R5A1_UNORM_PACK16, CompatibilityClass::BIT16, 4);
    define_image_format!(A1R5G5B5_UNORM_PACK16, CompatibilityClass::BIT16, 4);
    define_image_format!(R8_UNORM, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_SNORM, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_USCALED, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_SSCALED, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_UINT, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_SINT, CompatibilityClass::BIT8, 1);
    define_image_format!(R8_SRGB, CompatibilityClass::BIT8, 1);
    define_image_format!(R8G8_UNORM, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_SNORM, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_USCALED, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_SSCALED, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_UINT, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_SINT, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8_SRGB, CompatibilityClass::BIT16, 2);
    define_image_format!(R8G8B8_UNORM, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_SNORM, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_USCALED, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_SSCALED, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_UINT, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_SINT, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8_SRGB, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_UNORM, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_SNORM, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_USCALED, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_SSCALED, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_UINT, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_SINT, CompatibilityClass::BIT24, 3);
    define_image_format!(B8G8R8_SRGB, CompatibilityClass::BIT24, 3);
    define_image_format!(R8G8B8A8_UNORM, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_SNORM, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_USCALED, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_SSCALED, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_UINT, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_SINT, CompatibilityClass::BIT32, 4);
    define_image_format!(R8G8B8A8_SRGB, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_UNORM, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_SNORM, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_USCALED, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_SSCALED, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_UINT, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_SINT, CompatibilityClass::BIT32, 4);
    define_image_format!(B8G8R8A8_SRGB, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_UNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_SNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_USCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_SSCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_UINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_SINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A8B8G8R8_SRGB_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_UNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_SNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_USCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_SSCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_UINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2R10G10B10_SINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_UNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_SNORM_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_USCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_SSCALED_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_UINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(A2B10G10R10_SINT_PACK32, CompatibilityClass::BIT32, 4);
    define_image_format!(R16_UNORM, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_SNORM, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_USCALED, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_SSCALED, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_UINT, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_SINT, CompatibilityClass::BIT16, 1);
    define_image_format!(R16_SFLOAT, CompatibilityClass::BIT16, 1);
    define_image_format!(R16G16_UNORM, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_SNORM, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_USCALED, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_SSCALED, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_UINT, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_SINT, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16_SFLOAT, CompatibilityClass::BIT32, 2);
    define_image_format!(R16G16B16_UNORM, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_SNORM, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_USCALED, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_SSCALED, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_UINT, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_SINT, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16_SFLOAT, CompatibilityClass::BIT48, 3);
    define_image_format!(R16G16B16A16_UNORM, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_SNORM, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_USCALED, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_SSCALED, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_UINT, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_SINT, CompatibilityClass::BIT64, 4);
    define_image_format!(R16G16B16A16_SFLOAT, CompatibilityClass::BIT64, 4);
    define_image_format!(R32_UINT, CompatibilityClass::BIT32, 1);
    define_image_format!(R32_SINT, CompatibilityClass::BIT32, 1);
    define_image_format!(R32_SFLOAT, CompatibilityClass::BIT32, 1);
    define_image_format!(R32G32_UINT, CompatibilityClass::BIT64, 2);
    define_image_format!(R32G32_SINT, CompatibilityClass::BIT64, 2);
    define_image_format!(R32G32_SFLOAT, CompatibilityClass::BIT64, 2);
    define_image_format!(R32G32B32_UINT, CompatibilityClass::BIT96, 3);
    define_image_format!(R32G32B32_SINT, CompatibilityClass::BIT96, 3);
    define_image_format!(R32G32B32_SFLOAT, CompatibilityClass::BIT96, 3);
    define_image_format!(R32G32B32A32_UINT, CompatibilityClass::BIT128, 4);
    define_image_format!(R32G32B32A32_SINT, CompatibilityClass::BIT128, 4);
    define_image_format!(R32G32B32A32_SFLOAT, CompatibilityClass::BIT128, 4);
    define_image_format!(R64_UINT, CompatibilityClass::BIT64, 1);
    define_image_format!(R64_SINT, CompatibilityClass::BIT64, 1);
    define_image_format!(R64_SFLOAT, CompatibilityClass::BIT64, 1);
    define_image_format!(R64G64_UINT, CompatibilityClass::BIT128, 2);
    define_image_format!(R64G64_SINT, CompatibilityClass::BIT128, 2);
    define_image_format!(R64G64_SFLOAT, CompatibilityClass::BIT128, 2);
    define_image_format!(R64G64B64_UINT, CompatibilityClass::BIT192, 3);
    define_image_format!(R64G64B64_SINT, CompatibilityClass::BIT192, 3);
    define_image_format!(R64G64B64_SFLOAT, CompatibilityClass::BIT192, 3);
    define_image_format!(R64G64B64A64_UINT, CompatibilityClass::BIT256, 4);
    define_image_format!(R64G64B64A64_SINT, CompatibilityClass::BIT256, 4);
    define_image_format!(R64G64B64A64_SFLOAT, CompatibilityClass::BIT256, 4);
    define_image_format!(B10G11R11_UFLOAT_PACK32, CompatibilityClass::BIT32, 3);
    define_image_format!(E5B9G9R9_UFLOAT_PACK32, CompatibilityClass::BIT32, 3);
    define_image_format!(D16_UNORM, CompatibilityClass::D16, 1);
    define_image_format!(X8_D24_UNORM_PACK32, CompatibilityClass::D24, 1);
    define_image_format!(D32_SFLOAT, CompatibilityClass::D32, 1);
    define_image_format!(S8_UINT, CompatibilityClass::S8, 1);
    define_image_format!(D16_UNORM_S8_UINT, CompatibilityClass::D16S8, 2);
    define_image_format!(D24_UNORM_S8_UINT, CompatibilityClass::D24S8, 2);
    define_image_format!(D32_SFLOAT_S8_UINT, CompatibilityClass::D32S8, 2);
    define_image_format!(BC1_RGB_UNORM_BLOCK, CompatibilityClass::BC1_RGB, 3);
    define_image_format!(BC1_RGB_SRGB_BLOCK, CompatibilityClass::BC1_RGB, 3);
    define_image_format!(BC1_RGBA_UNORM_BLOCK, CompatibilityClass::BC1_RGBA, 4);
    define_image_format!(BC1_RGBA_SRGB_BLOCK, CompatibilityClass::BC1_RGBA, 4);
    define_image_format!(BC2_UNORM_BLOCK, CompatibilityClass::BC2, 4);
    define_image_format!(BC2_SRGB_BLOCK, CompatibilityClass::BC2, 4);
    define_image_format!(BC3_UNORM_BLOCK, CompatibilityClass::BC3, 4);
    define_image_format!(BC3_SRGB_BLOCK, CompatibilityClass::BC3, 4);
    define_image_format!(BC4_UNORM_BLOCK, CompatibilityClass::BC4, 1);
    define_image_format!(BC4_SNORM_BLOCK, CompatibilityClass::BC4, 1);
    define_image_format!(BC5_UNORM_BLOCK, CompatibilityClass::BC5, 2);
    define_image_format!(BC5_SNORM_BLOCK, CompatibilityClass::BC5, 2);
    define_image_format!(BC6H_UFLOAT_BLOCK, CompatibilityClass::BC6H, 3);
    define_image_format!(BC6H_SFLOAT_BLOCK, CompatibilityClass::BC6H, 3);
    define_image_format!(BC7_UNORM_BLOCK, CompatibilityClass::BC7, 4);
    define_image_format!(BC7_SRGB_BLOCK, CompatibilityClass::BC7, 4);
    define_image_format!(ETC2_R8G8B8_UNORM_BLOCK, CompatibilityClass::ETC2_RGB, 3);
    define_image_format!(ETC2_R8G8B8_SRGB_BLOCK, CompatibilityClass::ETC2_RGB, 3);
    define_image_format!(ETC2_R8G8B8A1_UNORM_BLOCK, CompatibilityClass::ETC2_RGBA, 4);
    define_image_format!(ETC2_R8G8B8A1_SRGB_BLOCK, CompatibilityClass::ETC2_RGBA, 4);
    define_image_format!(ETC2_R8G8B8A8_UNORM_BLOCK, CompatibilityClass::ETC2_EAC_RGBA, 4);
    define_image_format!(ETC2_R8G8B8A8_SRGB_BLOCK, CompatibilityClass::ETC2_EAC_RGBA, 4);
    define_image_format!(EAC_R11_UNORM_BLOCK, CompatibilityClass::EAC_R, 1);
    define_image_format!(EAC_R11_SNORM_BLOCK, CompatibilityClass::EAC_R, 1);
    define_image_format!(EAC_R11G11_UNORM_BLOCK, CompatibilityClass::EAC_RG, 2);
    define_image_format!(EAC_R11G11_SNORM_BLOCK, CompatibilityClass::EAC_RG, 2);
    define_image_format!(ASTC_4X4_UNORM_BLOCK, CompatibilityClass::ASTC_4X4, 4);
    define_image_format!(ASTC_4X4_SRGB_BLOCK, CompatibilityClass::ASTC_4X4, 4);
    define_image_format!(ASTC_5X4_UNORM_BLOCK, CompatibilityClass::ASTC_5X4, 4);
    define_image_format!(ASTC_5X4_SRGB_BLOCK, CompatibilityClass::ASTC_5X4, 4);
    define_image_format!(ASTC_5X5_UNORM_BLOCK, CompatibilityClass::ASTC_5X5, 4);
    define_image_format!(ASTC_5X5_SRGB_BLOCK, CompatibilityClass::ASTC_5X5, 4);
    define_image_format!(ASTC_6X5_UNORM_BLOCK, CompatibilityClass::ASTC_6X5, 4);
    define_image_format!(ASTC_6X5_SRGB_BLOCK, CompatibilityClass::ASTC_6X5, 4);
    define_image_format!(ASTC_6X6_UNORM_BLOCK, CompatibilityClass::ASTC_6X6, 4);
    define_image_format!(ASTC_6X6_SRGB_BLOCK, CompatibilityClass::ASTC_6X6, 4);
    define_image_format!(ASTC_8X5_UNORM_BLOCK, CompatibilityClass::ASTC_8X5, 4);
    define_image_format!(ASTC_8X5_SRGB_BLOCK, CompatibilityClass::ASTC_8X5, 4);
    define_image_format!(ASTC_8X6_UNORM_BLOCK, CompatibilityClass::ASTC_8X6, 4);
    define_image_format!(ASTC_8X6_SRGB_BLOCK, CompatibilityClass::ASTC_8X6, 4);
    define_image_format!(ASTC_8X8_UNORM_BLOCK, CompatibilityClass::ASTC_8X8, 4);
    define_image_format!(ASTC_8X8_SRGB_BLOCK, CompatibilityClass::ASTC_8X8, 4);
    define_image_format!(ASTC_10X5_UNORM_BLOCK, CompatibilityClass::ASTC_10X5, 4);
    define_image_format!(ASTC_10X5_SRGB_BLOCK, CompatibilityClass::ASTC_10X5, 4);
    define_image_format!(ASTC_10X6_UNORM_BLOCK, CompatibilityClass::ASTC_10X6, 4);
    define_image_format!(ASTC_10X6_SRGB_BLOCK, CompatibilityClass::ASTC_10X6, 4);
    define_image_format!(ASTC_10X8_UNORM_BLOCK, CompatibilityClass::ASTC_10X8, 4);
    define_image_format!(ASTC_10X8_SRGB_BLOCK, CompatibilityClass::ASTC_10X8, 4);
    define_image_format!(ASTC_10X10_UNORM_BLOCK, CompatibilityClass::ASTC_10X10, 4);
    define_image_format!(ASTC_10X10_SRGB_BLOCK, CompatibilityClass::ASTC_10X10, 4);
    define_image_format!(ASTC_12X10_UNORM_BLOCK, CompatibilityClass::ASTC_12X10, 4);
    define_image_format!(ASTC_12X10_SRGB_BLOCK, CompatibilityClass::ASTC_12X10, 4);
    define_image_format!(ASTC_12X12_UNORM_BLOCK, CompatibilityClass::ASTC_12X12, 4);
    define_image_format!(ASTC_12X12_SRGB_BLOCK, CompatibilityClass::ASTC_12X12, 4);
    define_image_format!(G8B8G8R8_422_UNORM, CompatibilityClass::BIT32_G8B8G8R8, 4);
    define_image_format!(B8G8R8G8_422_UNORM, CompatibilityClass::BIT32_B8G8R8G8, 4);
    define_image_format!(G8_B8_R8_3PLANE_420_UNORM, CompatibilityClass::PLANE3_8BIT_420, 3);
    define_image_format!(G8_B8R8_2PLANE_420_UNORM, CompatibilityClass::PLANE2_8BIT_420, 3);
    define_image_format!(G8_B8_R8_3PLANE_422_UNORM, CompatibilityClass::PLANE3_8BIT_422, 3);
    define_image_format!(G8_B8R8_2PLANE_422_UNORM, CompatibilityClass::PLANE2_8BIT_422, 3);
    define_image_format!(G8_B8_R8_3PLANE_444_UNORM, CompatibilityClass::PLANE3_8BIT_444, 3);
    define_image_format!(R10X6_UNORM_PACK16, CompatibilityClass::BIT16, 1);
    define_image_format!(R10X6G10X6_UNORM_2PACK16, CompatibilityClass::BIT32, 2);
    define_image_format!(R10X6G10X6B10X6A10X6_UNORM_4PACK16, CompatibilityClass::BIT64_R10G10B10A10, 4);
    define_image_format!(G10X6B10X6G10X6R10X6_422_UNORM_4PACK16, CompatibilityClass::BIT64_G10B10G10R10, 4);
    define_image_format!(B10X6G10X6R10X6G10X6_422_UNORM_4PACK16, CompatibilityClass::BIT64_B10G10R10G10, 4);
    define_image_format!(G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16, CompatibilityClass::PLANE3_10BIT_420, 3);
    define_image_format!(G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16, CompatibilityClass::PLANE2_10BIT_420, 3);
    define_image_format!(G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16, CompatibilityClass::PLANE3_10BIT_422, 3);
    define_image_format!(G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16, CompatibilityClass::PLANE2_10BIT_422, 3);
    define_image_format!(G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16, CompatibilityClass::PLANE3_10BIT_444, 3);
    define_image_format!(R12X4_UNORM_PACK16, CompatibilityClass::BIT16, 1);
    define_image_format!(R12X4G12X4_UNORM_2PACK16, CompatibilityClass::BIT32, 2);
    define_image_format!(R12X4G12X4B12X4A12X4_UNORM_4PACK16, CompatibilityClass::BIT64_R12G12B12A12, 4);
    define_image_format!(G12X4B12X4G12X4R12X4_422_UNORM_4PACK16, CompatibilityClass::BIT64_G12B12G12R12, 4);
    define_image_format!(B12X4G12X4R12X4G12X4_422_UNORM_4PACK16, CompatibilityClass::BIT64_B12G12R12G12, 4);
    define_image_format!(G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16, CompatibilityClass::PLANE3_12BIT_420, 3);
    define_image_format!(G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16, CompatibilityClass::PLANE2_12BIT_420, 3);
    define_image_format!(G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16, CompatibilityClass::PLANE3_12BIT_422, 3);
    define_image_format!(G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16, CompatibilityClass::PLANE2_12BIT_422, 3);
    define_image_format!(G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16, CompatibilityClass::PLANE3_12BIT_444, 3);
    define_image_format!(G16B16G16R16_422_UNORM, CompatibilityClass::BIT64_G16B16G16R16, 3);
    define_image_format!(B16G16R16G16_422_UNORM, CompatibilityClass::BIT64_B16G16R16G16, 3);
    define_image_format!(G16_B16_R16_3PLANE_420_UNORM, CompatibilityClass::PLANE3_16BIT_420, 3);
    define_image_format!(G16_B16R16_2PLANE_420_UNORM, CompatibilityClass::PLANE2_16BIT_420, 3);
    define_image_format!(G16_B16_R16_3PLANE_422_UNORM, CompatibilityClass::PLANE3_16BIT_422, 3);
    define_image_format!(G16_B16R16_2PLANE_422_UNORM, CompatibilityClass::PLANE2_16BIT_422, 3);
    define_image_format!(G16_B16_R16_3PLANE_444_UNORM, CompatibilityClass::PLANE3_16BIT_444, 3);
}

impl PartialEq for ImageFormat {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
    }
}

impl Debug for ImageFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageFormat").field("format", &self.format).finish()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ImageSize {
    Type1D { width: u32, mip_levels: u32, array_layers: u32 },
    Type2D { width: u32, height: u32, mip_levels: u32, array_layers: u32 },
    Type3D { width: u32, height: u32, depth: u32, mip_levels: u32 },
}

impl ImageSize {
    pub const fn make_1d(width: u32) -> Self {
        ImageSize::Type1D { width, mip_levels: 1, array_layers: 1 }
    }

    pub const fn make_1d_mip(width: u32, mip_levels: u32) -> Self {
        ImageSize::Type1D { width, mip_levels, array_layers: 1 }
    }

    pub const fn make_1d_array(width: u32, array_layers: u32) -> Self {
        ImageSize::Type1D { width, mip_levels: 1, array_layers }
    }

    pub const fn make_1d_array_mip(width: u32, array_layers: u32, mip_levels: u32) -> Self {
        ImageSize::Type1D { width, mip_levels, array_layers }
    }

    pub const fn make_2d(width: u32, height: u32) -> Self {
        ImageSize::Type2D { width, height, mip_levels: 1, array_layers: 1 }
    }

    pub const fn make_2d_mip(width: u32, height: u32, mip_levels: u32) -> Self {
        ImageSize::Type2D { width, height, mip_levels, array_layers: 1 }
    }

    pub const fn make_2d_array(width: u32, height: u32, array_layers: u32) -> Self {
        ImageSize::Type2D { width, height, mip_levels: 1, array_layers }
    }

    pub const fn make_2d_array_mip(width: u32, height: u32, array_layers: u32, mip_levels: u32) -> Self {
        ImageSize::Type2D { width, height, mip_levels, array_layers }
    }

    pub const fn make_3d(width: u32, height: u32, depth: u32) -> Self {
        ImageSize::Type3D { width, height, depth, mip_levels: 1 }
    }

    pub const fn make_3d_mip(width: u32, height: u32, depth: u32, mip_levels: u32) -> Self {
        ImageSize::Type3D { width, height, depth, mip_levels }
    }

    pub const fn get_width(&self) -> u32 {
        match self {
            ImageSize::Type1D { width, .. } => *width,
            ImageSize::Type2D { width, .. } => *width,
            ImageSize::Type3D { width, .. } => *width
        }
    }

    pub const fn get_height(&self) -> u32 {
        match self {
            ImageSize::Type1D { .. } => 1,
            ImageSize::Type2D { height, .. } => *height,
            ImageSize::Type3D { height, .. } => *height
        }
    }

    pub const fn get_depth(&self) -> u32 {
        match self {
            ImageSize::Type1D { .. } => 1,
            ImageSize::Type2D { .. } => 1,
            ImageSize::Type3D { depth, .. } => *depth
        }
    }

    pub const fn get_array_layers(&self) -> u32 {
        match self {
            ImageSize::Type1D { array_layers, .. } => *array_layers,
            ImageSize::Type2D { array_layers, .. } => *array_layers,
            ImageSize::Type3D { .. } => 1,
        }
    }

    pub const fn get_mip_levels(&self) -> u32 {
        match self {
            ImageSize::Type1D { mip_levels, .. } => *mip_levels,
            ImageSize::Type2D { mip_levels, .. } => *mip_levels,
            ImageSize::Type3D { mip_levels, .. } => *mip_levels,
        }
    }

    pub const fn as_extent_3d(&self) -> ash::vk::Extent3D {
        match self {
            ImageSize::Type1D { width, .. } => ash::vk::Extent3D { width: *width, height: 1, depth: 1 },
            ImageSize::Type2D { width, height, .. } => ash::vk::Extent3D { width: *width, height: *height, depth: 1 },
            ImageSize::Type3D { width, height, depth, .. } => ash::vk::Extent3D { width: *width, height: *height, depth: *depth }
        }
    }

    pub fn fill_extent_3d(&self, extent: &mut ash::vk::Extent3D) {
        *extent = self.as_extent_3d();
    }
}

#[derive(Copy, Clone)]
pub struct ImageSpec {
    pub format: &'static ImageFormat,
    pub sample_count: ash::vk::SampleCountFlags,
    pub size: ImageSize,
}

impl ImageSpec {
    pub const fn new(size: ImageSize, format: &'static ImageFormat, sample_count: ash::vk::SampleCountFlags) -> Self {
        ImageSpec { format, size, sample_count }
    }

    pub const fn get_size(&self) -> ImageSize {
        self.size
    }

    pub const fn borrow_size(&self) -> &ImageSize {
        &self.size
    }

    pub const fn get_format(&self) -> &'static ImageFormat {
        self.format
    }

    pub const fn get_sample_count(&self) -> ash::vk::SampleCountFlags {
        self.sample_count
    }
}