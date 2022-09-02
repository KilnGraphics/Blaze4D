use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ash::vk;
use crate::allocator::{Allocation, HostAccess};

use super::share::Share2;
use crate::define_uuid_type;

use crate::prelude::*;

macro_rules! id_type {
    ($name: ident, $id_func: expr) => {
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                $id_func(self).eq(&$id_func(other))
            }
        }

        impl Eq for $name {
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                $id_func(self).partial_cmp(&$id_func(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                $id_func(self).cmp(&$id_func(other))
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                $id_func(self).hash(state)
            }
        }
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BufferInfo {
    pub size: vk::DeviceSize,
}

define_uuid_type!(pub, BufferId);

pub struct Buffer {
    share: Arc<Share2>,
    id: BufferId,
    info: BufferInfo,
    handle: vk::Buffer,
    alloc_type: BufferAllocationType,
}

impl Buffer {
    pub(super) fn new_persistent(share: Arc<Share2>, size: vk::DeviceSize) -> Self {
        let id = BufferId::new();

        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (handle, allocation, _) = unsafe {
            share.get_device().get_allocator().create_buffer(&info, HostAccess::None, &format_args!(""))
        }.expect("Failed to create persistent buffer");

        Self {
            share,
            id,
            info: BufferInfo {
                size
            },
            handle,
            alloc_type: BufferAllocationType::Persistent(allocation),
        }
    }

    pub fn get_id(&self) -> BufferId {
        self.id
    }

    pub fn get_info(&self) -> &BufferInfo {
        &self.info
    }

    pub(super) fn get_handle(&self) -> vk::Buffer {
        self.handle
    }

    pub(super) fn get_offset(&self) -> vk::DeviceSize {
        0
    }
}

id_type!(Buffer, Buffer::get_id);

impl Drop for Buffer {
    fn drop(&mut self) {
        match self.alloc_type {
            BufferAllocationType::Persistent(allocation) => unsafe {
                self.share.get_device().get_allocator().destroy_buffer(self.handle, allocation)
            },
        }
    }
}

pub(super) enum BufferAllocationType {
    Persistent(Allocation),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ImageSize {
    Type1D {
        size: u32,
        mip_levels: u32,
        array_layers: u32,
    },
    Type2D {
        size: Vec2u32,
        mip_levels: u32,
        array_layers: u32,
    },
    Type3D {
        size: Vec3u32,
        mip_levels: u32,
    },
}

impl ImageSize {
    pub fn new_1d(size: u32, mip_levels: u32, array_layers: u32) -> Self {
        Self::Type1D {
            size,
            mip_levels,
            array_layers
        }
    }

    pub fn new_2d(size: Vec2u32, mip_levels: u32, array_layers: u32) -> Self {
        Self::Type2D {
            size,
            mip_levels,
            array_layers
        }
    }

    pub fn new_3d(size: Vec3u32, mip_levels: u32) -> Self {
        Self::Type3D {
            size,
            mip_levels
        }
    }

    pub fn get_size_as_vec3(&self) -> Vec3u32 {
        match self {
            ImageSize::Type1D { size, .. } => Vec3u32::new(*size, 1, 1),
            ImageSize::Type2D { size, .. } => Vec3u32::new(size[0], size[1], 1),
            ImageSize::Type3D { size, .. } => *size,
        }
    }

    pub fn get_vk_extent3d(&self) -> vk::Extent3D {
        let size = self.get_size_as_vec3();
        vk::Extent3D {
            width: size[0],
            height: size[1],
            depth: size[2],
        }
    }

    pub fn get_mip_levels(&self) -> u32 {
        match self {
            ImageSize::Type1D { mip_levels, .. } |
            ImageSize::Type2D { mip_levels, .. } |
            ImageSize::Type3D { mip_levels, .. } => *mip_levels,
        }
    }

    pub fn get_array_layers(&self) -> u32 {
        match self {
            ImageSize::Type1D { array_layers, .. } |
            ImageSize::Type2D { array_layers, .. } => *array_layers,
            ImageSize::Type3D { .. } => 1,
        }
    }

    pub fn is_1d(&self) -> bool {
        match self {
            ImageSize::Type1D { .. } => true,
            _ => false,
        }
    }

    pub fn is_2d(&self) -> bool {
        match self {
            ImageSize::Type2D { .. } => true,
            _ => false,
        }
    }

    pub fn is_3d(&self) -> bool {
        match self {
            ImageSize::Type3D { .. } => true,
            _ => false,
        }
    }

    pub fn get_vk_image_type(&self) -> vk::ImageType {
        match self {
            ImageSize::Type1D { .. } => vk::ImageType::TYPE_1D,
            ImageSize::Type2D { .. } => vk::ImageType::TYPE_2D,
            ImageSize::Type3D { .. } => vk::ImageType::TYPE_3D,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ImageInfo {
    pub size: ImageSize,
    pub format: vk::Format,
    pub aspect_mask: vk::ImageAspectFlags,
}

impl ImageInfo {
    pub fn get_full_subresource_range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: self.aspect_mask,
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS
        }
    }

    pub fn get_full_subresource_layers(&self, mip_level: u32) -> vk::ImageSubresourceLayers {
        vk::ImageSubresourceLayers {
            aspect_mask: self.aspect_mask,
            mip_level,
            base_array_layer: 0,
            layer_count: self.size.get_array_layers()
        }
    }
}

define_uuid_type!(pub, ImageId);

pub struct Image {
    share: Arc<Share2>,
    id: ImageId,
    info: ImageInfo,
    handle: vk::Image,
    allocation: Allocation,
    view: vk::ImageView,
    initialized: AtomicBool,
}

impl Image {
    pub(super) fn new_persistent_color(share: Arc<Share2>, format: vk::Format, size: ImageSize) -> Self {
        let id = ImageId::new();

        let info = vk::ImageCreateInfo::builder()
            .image_type(size.get_vk_image_type())
            .format(format)
            .extent(size.get_vk_extent3d())
            .mip_levels(size.get_mip_levels())
            .array_layers(size.get_array_layers())
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (handle, allocation, _) = unsafe {
            share.get_device().get_allocator().create_image(&info, HostAccess::None, &format_args!(""))
        }.expect("Failed to create persistent image");

        let image_info = ImageInfo {
            size,
            format,
            aspect_mask: vk::ImageAspectFlags::COLOR,
        };

        let info = vk::ImageViewCreateInfo::builder()
            .image(handle)
            .view_type(Self::get_base_image_view_type(&size))
            .format(image_info.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(image_info.get_full_subresource_range());

        let view = unsafe {
            share.get_device().vk().create_image_view(&info, None)
        }.map_err(|err| {
            unsafe { share.get_device().get_allocator().destroy_image(handle, allocation) };
            err
        }).expect("Failed to create image view for persistent image");

        Self {
            share,
            id,
            info: image_info,
            handle,
            allocation,
            view,
            initialized: AtomicBool::from(false),
        }
    }

    pub fn get_id(&self) -> ImageId {
        self.id
    }

    pub fn get_info(&self) -> &ImageInfo {
        &self.info
    }

    pub(super) fn get_handle(&self) -> vk::Image {
        self.handle
    }

    pub(super) fn get_default_view_handle(&self) -> vk::ImageView {
        self.view
    }

    pub(super) fn get_initialized(&self) -> &AtomicBool {
        &self.initialized
    }

    fn get_base_image_view_type(size: &ImageSize) -> vk::ImageViewType {
        match (size.get_vk_image_type(), size.get_array_layers()) {
            (vk::ImageType::TYPE_1D, 1) => vk::ImageViewType::TYPE_1D,
            (vk::ImageType::TYPE_1D, _) => vk::ImageViewType::TYPE_1D_ARRAY,
            (vk::ImageType::TYPE_2D, 1) => vk::ImageViewType::TYPE_2D,
            (vk::ImageType::TYPE_2D, _) => vk::ImageViewType::TYPE_2D_ARRAY,
            (vk::ImageType::TYPE_3D, _) => vk::ImageViewType::TYPE_3D,
            _ => panic!("Invalid image type"),
        }
    }
}

id_type!(Image, Image::get_id);

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.share.get_device().vk().destroy_image_view(self.view, None);
            self.share.get_device().get_allocator().destroy_image(self.handle, self.allocation);
        }
    }
}