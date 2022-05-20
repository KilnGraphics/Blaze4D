#[macro_use]
extern crate static_assertions;

pub mod device;
pub mod instance;
pub mod objects;

pub mod vk;
pub mod util;
pub mod b4d;

pub mod debug;

pub use util::id::UUID;
pub use util::id::NamedUUID;

mod glfw_surface;
mod renderer;
pub mod window;


pub mod prelude {
    pub use crate::UUID;
    pub use crate::NamedUUID;

    pub use crate::instance::instance::InstanceContext;
    pub use crate::device::device::DeviceContext;
    pub use crate::device::device::DeviceEnvironment;

    pub type Vec2f32 = nalgebra::Vector2<f32>;
    pub type Vec3f32 = nalgebra::Vector3<f32>;
    pub type Vec4f32 = nalgebra::Vector4<f32>;

    pub type Vec2u32 = nalgebra::Vector2<u32>;
    pub type Vec3u32 = nalgebra::Vector3<u32>;
    pub type Vec4u32 = nalgebra::Vector4<u32>;

    pub type Vec2i32 = nalgebra::Vector2<i32>;
    pub type Vec3i32 = nalgebra::Vector3<i32>;
    pub type Vec4i32 = nalgebra::Vector4<i32>;

    pub type Mat4f32 = nalgebra::Matrix4<f32>;
}