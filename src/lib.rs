extern crate core;

pub mod vk;
pub mod util;
pub mod b4d;

pub use util::id::UUID;
pub use util::id::NamedUUID;

mod glfw_surface;
mod renderer;
pub mod window;
mod debug;

pub mod prelude {
    pub type Vec2f32 = nalgebra::Vector2<f32>;
    pub type Vec3f32 = nalgebra::Vector3<f32>;

    pub type Vec2u32 = nalgebra::Vector2<u32>;
}