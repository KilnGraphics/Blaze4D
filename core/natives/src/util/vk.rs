use ash::vk;

use crate::prelude::*;

#[inline]
pub fn make_full_viewport(size: Vec2u32) -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: size[0] as f32,
        height: size[1] as f32,
        min_depth: 0.0,
        max_depth: 1.0
    }
}

#[inline]
pub fn make_full_rect(size: Vec2u32) -> vk::Rect2D {
    vk::Rect2D {
        offset: vk::Offset2D{ x: 0, y: 0 },
        extent: vk::Extent2D{ width: size[0], height: size[1] }
    }
}