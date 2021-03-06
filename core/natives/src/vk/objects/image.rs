use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use ash::vk;
use ash::vk::Handle;
use crate::objects::id::{ImageId, ObjectId};
use crate::vk::objects::Format;

use crate::prelude::*;


#[derive(Copy, Clone)]
pub struct Image {
    id: ImageId,
    handle: vk::Image,
}

impl Image {
    pub fn new(handle: vk::Image) -> Self {
        Self {
            id: ImageId::new(),
            handle,
        }
    }

    pub fn get_id(&self) -> ImageId {
        self.id
    }

    pub fn get_handle(&self) -> vk::Image {
        self.handle
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Image {
}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for Image {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("vkImage({:#016X}, {:#016X})", self.id.get_raw(), self.handle.as_raw()))
    }
}

impl From<Image> for ImageId {
    fn from(image: Image) -> Self {
        image.get_id()
    }
}

impl From<Image> for UUID {
    fn from(image: Image) -> Self {
        image.get_id().as_uuid()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub const fn get_vulkan_type(&self) -> vk::ImageType {
        match self {
            ImageSize::Type1D { .. } => vk::ImageType::TYPE_1D,
            ImageSize::Type2D { .. } => vk::ImageType::TYPE_2D,
            ImageSize::Type3D { .. } => vk::ImageType::TYPE_3D,
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSpec {
    pub format: &'static Format,
    pub sample_count: ash::vk::SampleCountFlags,
    pub size: ImageSize,
}

impl ImageSpec {
    pub const fn new(size: ImageSize, format: &'static Format, sample_count: vk::SampleCountFlags) -> Self {
        ImageSpec { format, size, sample_count }
    }

    pub const fn new_single_sample(size: ImageSize, format: &'static Format) -> Self {
        ImageSpec { format, size, sample_count: vk::SampleCountFlags::TYPE_1 }
    }

    pub const fn get_size(&self) -> ImageSize {
        self.size
    }

    pub const fn borrow_size(&self) -> &ImageSize {
        &self.size
    }

    pub const fn get_format(&self) -> &'static Format {
        self.format
    }

    pub const fn get_sample_count(&self) -> ash::vk::SampleCountFlags {
        self.sample_count
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ash::vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub mip_level_count: u32,
    pub base_array_layer: u32,
    pub array_layer_count: u32,
}

impl ImageSubresourceRange {
    pub fn full_color() -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            mip_level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            array_layer_count: vk::REMAINING_ARRAY_LAYERS,
        }
    }

    pub const fn as_vk_subresource_range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: self.aspect_mask,
            base_mip_level: self.base_mip_level,
            level_count: self.mip_level_count,
            base_array_layer: self.base_array_layer,
            layer_count: self.array_layer_count
        }
    }
}

impl From<ImageSubresourceRange> for ash::vk::ImageSubresourceRange {
    fn from(src: ImageSubresourceRange) -> Self {
        ash::vk::ImageSubresourceRange::builder()
            .aspect_mask(src.aspect_mask)
            .base_mip_level(src.base_mip_level)
            .base_array_layer(src.base_array_layer)
            .layer_count(src.array_layer_count)
            .build()
    }
}

/// Contains a description for a vulkan image.
///
/// This only contains static information relevant to vulkan (i.e. size or supported usage flags).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageDescription {
    pub spec: ImageSpec,
    pub usage_flags: vk::ImageUsageFlags,
}

impl ImageDescription {
    pub fn new_simple(spec: ImageSpec, usage: vk::ImageUsageFlags) -> Self {
        Self{ spec, usage_flags: usage }
    }
}

/// Contains a description for a vulkan image view.
///
/// This only contains static information relevant to vulkan (i.e. range or format, however not the
/// source image as image views with different sources may have the same description).
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub struct ImageViewDescription {
    pub view_type: vk::ImageViewType,
    pub format: &'static Format,
    pub components: vk::ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl ImageViewDescription {
    /// Creates a image view description with identity component mapping and subresource range
    /// covering all mip levels and array layers.
    pub fn make_full(view_type: vk::ImageViewType, format: &'static Format, aspect_mask: vk::ImageAspectFlags) -> Self {
        Self {
            view_type,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            },
            subresource_range: ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                mip_level_count: vk::REMAINING_MIP_LEVELS,
                base_array_layer: 0,
                array_layer_count: vk::REMAINING_ARRAY_LAYERS,
            }
        }
    }

    /// Creates a image view description with identity component mapping
    pub fn make_range(view_type: vk::ImageViewType, format: &'static Format, subresource_range: ImageSubresourceRange) -> Self {
        Self {
            view_type,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            },
            subresource_range
        }
    }
}

pub struct ImageInstanceData {
    handle: vk::Image
}

impl ImageInstanceData {
    pub fn new(handle: vk::Image) -> Self {
        Self {
            handle
        }
    }

    pub unsafe fn get_handle(&self) -> vk::Image {
        self.handle
    }
}

pub struct ImageViewInstanceData {
    handle: vk::ImageView,
    source_image: crate::vk::objects::types::ImageId,
}

impl ImageViewInstanceData {
    pub fn new(handle: vk::ImageView, source_image: crate::vk::objects::types::ImageId) -> Self {
        Self {
            handle,
            source_image,
        }
    }

    pub fn get_source_image(&self) -> crate::vk::objects::types::ImageId {
        self.source_image
    }

    pub unsafe fn get_handle(&self) -> vk::ImageView {
        self.handle
    }
}