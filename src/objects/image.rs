use std::fmt::{Debug, Formatter};
use std::hash::Hasher;
use twox_hash::XxHash64;

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
        CompatibilityClass{ name }
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
    define_compatibility_class!(ASTC_4x4);
    define_compatibility_class!(ASTC_5x4);
    define_compatibility_class!(ASTC_5x5);
    define_compatibility_class!(ASTC_6x5);
    define_compatibility_class!(ASTC_6x6);
    define_compatibility_class!(ASTC_8x5);
    define_compatibility_class!(ASTC_8x6);
    define_compatibility_class!(ASTC_8x8);
    define_compatibility_class!(ASTC_10x5);
    define_compatibility_class!(ASTC_10x6);
    define_compatibility_class!(ASTC_10x8);
    define_compatibility_class!(ASTC_10x10);
    define_compatibility_class!(ASTC_12x10);
    define_compatibility_class!(ASTC_12x12);
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

pub struct ImageFormat {
    format: ash::vk::Format,
    compatibility_class: CompatibilityClass,
}

impl ImageFormat {
    pub const fn new(format: ash::vk::Format, compatibility_class: CompatibilityClass) -> Self {
        ImageFormat{ format, compatibility_class }
    }

    pub const fn get_format(&self) -> ash::vk::Format {
        self.format
    }


}