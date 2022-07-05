#[macro_use]
extern crate static_assertions;

use std::fmt::{Debug, Display, Formatter};

pub mod device;
pub mod instance;
pub mod objects;
pub mod renderer;

pub mod vk;
pub mod util;
pub mod b4d;

mod glfw_surface;
pub mod window;
mod c_api;
mod c_log;

pub struct BuildInfo {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub dev_build: bool,
}

impl Debug for BuildInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.dev_build {
            f.write_fmt(format_args!("{}({}.{}.{}-DEVELOPMENT)", CRATE_NAME, self.version_major, self.version_minor, self.version_patch))
        } else {
            f.write_fmt(format_args!("{}({}.{}.{})", CRATE_NAME, self.version_major, self.version_minor, self.version_patch))
        }
    }
}

pub const CRATE_NAME: &'static str = "Blaze4D-Core";
pub const BUILD_INFO: BuildInfo = BuildInfo {
    version_major: 0,
    version_minor: 1,
    version_patch: 0,
    dev_build: option_env!("B4D_RELEASE_BUILD").is_none(),
};

pub mod prelude {
    pub use crate::util::id::UUID;
    pub use crate::util::id::NamedUUID;

    pub use crate::instance::instance::InstanceContext;
    pub use crate::device::device::DeviceFunctions;
    pub use crate::device::device::Queue;
    pub use crate::device::device::DeviceContext;

    pub type Vec2f32 = nalgebra::Vector2<f32>;
    pub type Vec3f32 = nalgebra::Vector3<f32>;
    pub type Vec4f32 = nalgebra::Vector4<f32>;

    pub type Vec2u32 = nalgebra::Vector2<u32>;
    pub type Vec3u32 = nalgebra::Vector3<u32>;
    pub type Vec4u32 = nalgebra::Vector4<u32>;

    pub type Vec2i32 = nalgebra::Vector2<i32>;
    pub type Vec3i32 = nalgebra::Vector3<i32>;
    pub type Vec4i32 = nalgebra::Vector4<i32>;

    pub type Mat2f32 = nalgebra::Matrix2<f32>;
    pub type Mat3f32 = nalgebra::Matrix3<f32>;
    pub type Mat4f32 = nalgebra::Matrix4<f32>;
}