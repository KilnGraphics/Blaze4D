use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::AtomicU64;

use ash::vk;
use crate::define_uuid_type;

use crate::device::device::Queue;
use crate::objects::sync::Semaphore;
use crate::renderer::emulator::{MeshData, PassId};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::renderer::emulator::share::Share;
use crate::renderer::emulator::staging::{StagingAllocationId, StagingMemoryPool};
use crate::renderer::emulator::worker::{GlobalImageWrite, GlobalMeshWrite, WorkerTask};
use crate::util::alloc::next_aligned;

define_uuid_type!(pub, GlobalMeshId);

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum GlobalObjectCreateError {
    Vulkan(vk::Result),
    Allocation,
}

impl From<vk::Result> for GlobalObjectCreateError {
    fn from(err: vk::Result) -> Self {
        GlobalObjectCreateError::Vulkan(err)
    }
}

pub struct GlobalMesh {
    share: Arc<Share>,
    id: GlobalMeshId,

    last_used_pass: AtomicU64,

    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    buffer_size: vk::DeviceSize,

    draw_info: GlobalMeshDrawInfo,
}

impl GlobalMesh {
    pub(super) fn new(share: Arc<Share>, data: &MeshData) -> Result<Arc<Self>, GlobalObjectCreateError> {
        let index_offset = next_aligned(data.vertex_data.len() as vk::DeviceSize, data.get_index_size() as vk::DeviceSize);
        let required_size = index_offset + (data.index_data.len() as vk::DeviceSize);

        let (buffer, allocation) = Self::create_buffer(share.get_device(), required_size)?;

        let (staging, staging_allocation) = share.get_staging_pool().lock().unwrap_or_else(|_| {
            log::error!("Poisoned staging memory mutex in GlobalMesh::new");
            panic!()
        }).allocate(required_size, 1);

        unsafe {
            let dst = std::slice::from_raw_parts_mut(mapped.as_ptr(), required_size as usize);

            dst[0..data.vertex_data.len()].copy_from_slice(data.vertex_data);
            dst[(index_offset as usize)..].copy_from_slice(data.index_data);
        }

        let draw_info = GlobalMeshDrawInfo {
            buffer,
            first_index: (index_offset / (data.get_index_size() as vk::DeviceSize)) as u32,
            index_type: data.index_type,
            index_count: data.index_count,
            primitive_topology: data.primitive_topology
        };

        let mesh = Arc::new(GlobalMesh {
            share,
            id: GlobalMeshId::new(),

            last_used_pass: AtomicU64::new(0),

            buffer,
            allocation: Some(allocation),
            buffer_size: required_size,

            draw_info
        });

        mesh.share.push_task(WorkerTask::WriteGlobalMesh(GlobalMeshWrite {
            after_pass: PassId::from_raw(0),
            staging_allocation,
            staging_range: (staging.offset, required_size),
            staging_buffer: staging.buffer,
            dst_mesh: mesh.clone(),
            regions: Box::new([vk::BufferCopy {
                src_offset: staging.offset,
                dst_offset: 0,
                size: required_size
            }])
        }, true));

        Ok(mesh)
    }

    pub(super) fn get_buffer_handle(&self) -> vk::Buffer {
        self.buffer
    }

    pub(super) fn get_draw_info(&self) -> &GlobalMeshDrawInfo {
        &self.draw_info
    }

    fn create_buffer(device: &DeviceContext, size: vk::DeviceSize) -> Result<(vk::Buffer, Allocation), GlobalObjectCreateError> {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateBuffer returned {:?} in GlobalMesh::create_buffer", err);
            err
        })?;

        let alloc = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuOnly).map_err(|_| {
            log::error!("Failed to allocate buffer memory in GlobalMesh::create_buffer");
            unsafe { device.vk().destroy_buffer(buffer, None) };
            GlobalObjectCreateError::Allocation
        })?;

        if let Err(err) = unsafe {
            device.vk().bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
        } {
            log::error!("vkBindBufferMemory returned {:?} in GlobalMesh::create_buffer", err);
            unsafe { device.vk().destroy_buffer(buffer, None) };
            device.get_allocator().free(alloc);
            return Err(GlobalObjectCreateError::Vulkan(err));
        }

        Ok((buffer, alloc))
    }
}

impl PartialEq for GlobalMesh {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for GlobalMesh {
}

impl PartialOrd for GlobalMesh {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for GlobalMesh {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for GlobalMesh {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Drop for GlobalMesh {
    fn drop(&mut self) {
        let allocation = self.allocation.take().unwrap();
        let device = self.share.get_device();

        unsafe {
            device.vk().destroy_buffer(self.buffer, None);
        }
        device.get_allocator().free(allocation);
    }
}

pub(super) struct GlobalMeshDrawInfo {
    pub(super) buffer: vk::Buffer,
    pub(super) first_index: u32,
    pub(super) index_count: u32,
    pub(super) index_type: vk::IndexType,
    pub(super) primitive_topology: vk::PrimitiveTopology,
}

pub struct ImageData<'a> {
    /// The image data
    pub data: &'a [u8],

    /// The stride between 2 rows of image data in texels. If 0 the data is assumed to be tightly packed.
    pub row_stride: u32,

    /// The offset of the upload region in the image.
    pub offset: Vec2u32,

    /// The size of the upload region in the image.
    pub extent: Vec2u32,
}

impl<'a> ImageData<'a> {
    pub fn new_full(data: &'a [u8], size: Vec2u32) -> Self {
        Self {
            data,
            row_stride: 0,
            offset: Vec2u32::new(0, 0),
            extent: size,
        }
    }

    pub fn new_full_with_stride(data: &'a [u8], row_stride: u32, size: Vec2u32) -> Self {
        Self {
            data,
            row_stride,
            offset: Vec2u32::new(0, 0),
            extent: size,
        }
    }

    pub fn new_extent(data: &'a [u8], offset: Vec2u32, extent: Vec2u32) -> Self {
        Self {
            data,
            row_stride: 0,
            offset,
            extent
        }
    }

    pub fn new_extent_with_stride(data: &'a [u8], row_stride: u32, offset: Vec2u32, extent: Vec2u32) -> Self {
        Self {
            data,
            row_stride,
            offset,
            extent
        }
    }
}

define_uuid_type!(pub, GlobalImageId);

pub struct GlobalImage {
    share: Arc<Share>,
    id: GlobalImageId,

    last_used_pass: AtomicU64,

    image: vk::Image,
    allocation: Option<Allocation>,
    size: Vec2u32,
    mip_levels: u32,
}

impl GlobalImage {
    pub(super) fn new(share: Arc<Share>, format: vk::Format, mip_levels: u32, data: &ImageData) -> Result<Arc<Self>, GlobalObjectCreateError> {
        assert_eq!(data.offset, Vec2u32(0, 0));

        let (image, allocation) = Self::create_image(share.get_device(), format, data.extent, mip_levels)?;

        let (staging, staging_allocation) = share.get_staging_pool().lock().unwrap_or_else(|_| {
            log::error!("Poisoned staging pool lock in GlobalImage::new");
            panic!()
        }).allocate(data.data.len() as u64, 1);

        unsafe {
            let dst = std::slice::from_raw_parts_mut(mapped.as_ptr(), data.data.len());
            dst.copy_from_slice(data.data);
        }

        let image = Arc::new(GlobalImage {
            share,
            id: GlobalImageId::new(),

            last_used_pass: AtomicU64::new(0),

            image,
            allocation: Some(allocation),
            size: data.extent,
            mip_levels
        });

        image.share.push_task(WorkerTask::WriteGlobalImage(GlobalImageWrite {
            after_pass: PassId::from_raw(0),
            staging_allocation,
            staging_range: (staging.offset, data.data.len() as u64),
            staging_buffer: staging.buffer,
            dst_image: image.clone(),
            regions: Box::new([vk::BufferImageCopy {
                buffer_offset: staging.offset,
                buffer_row_length: data.row_stride,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1
                },
                image_offset: vk::Offset3D{ x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D{ width: data.extent[0], height: data.extent[1], depth: 1 }
            }])
        }, true));

        Ok(image)
    }

    pub(super) fn get_image_handle(&self) -> vk::Image {
        self.image
    }

    pub(super) fn get_image_size(&self) -> Vec2u32 {
        self.size
    }

    pub(super) fn get_mip_levels(&self) -> u32 {
        self.mip_levels
    }

    pub(super) fn get_sampler_view(&self) -> vk::ImageView {
        todo!()
    }

    pub fn upload(&self, data: &[ImageData]) {
        if data.is_empty() {
            return;
        }

        if let Some(share) = self.share.upgrade() {
            todo!()
        }
    }

    fn create_image(device: &DeviceContext, format: vk::Format, size: Vec2u32, mip_levels: u32) -> Result<(vk::Image, Allocation), GlobalObjectCreateError> {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: size[0],
                height: size[1],
                depth: 1
            })
            .mip_levels(mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            device.vk().create_image(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateImage returned {:?} in GlobalImage::create_image", err);
            err
        })?;

        let allocation = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).map_err(|_| {
            log::error!("Failed to allocate image memory in GlobalImage::create_image");
            unsafe { device.vk().destroy_image(image, None) };
            GlobalObjectCreateError::Allocation
        })?;

        if let Err(err) = unsafe {
            device.vk().bind_image_memory(image, allocation.memory(), allocation.offset())
        } {
            log::error!("vkBindImageMemory returned {:?} in GlobalImage::create_image", err);
            unsafe { device.vk().destroy_image(image, None) };
            device.get_allocator().free(allocation);
            return Err(GlobalObjectCreateError::Vulkan(err));
        }

        Ok((image, allocation))
    }
}

impl PartialEq for GlobalImage {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for GlobalImage {
}

impl PartialOrd for GlobalImage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for GlobalImage {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for GlobalImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Drop for GlobalImage {
    fn drop(&mut self) {
        let device = self.share.get_device();
        unsafe {
            device.vk().destroy_image(self.image, None);
        }
        device.get_allocator().free(self.allocation.take().unwrap());
    }
}




struct StaticImage {
    image: vk::Image,
    allocation: Allocation,
    view: vk::ImageView,

    used_counter: u32,
    marked: bool,
}

impl StaticImage {
    /// Attempts to increment the used counter.
    ///
    /// If the image is marked the counter is not incremented and false is returned.
    fn inc(&mut self) -> bool {
        if self.marked {
            return false;
        }

        self.used_counter += 1;
        true
    }

    /// Decrements the used counter.
    ///
    /// If the image is marked and the counter decrements to 0 true is returned indicating that the
    /// image can be destroyed.
    fn dec(&mut self) -> bool {
        if self.used_counter == 0 {
            log::error!("Used counter is already 0 when calling StaticImage::dec");
            panic!()
        }

        self.used_counter -= 1;

        if self.marked && self.is_unused() {
            return true;
        }
        false
    }

    /// Returns true if the mesh used counter is 0
    fn is_unused(&self) -> bool {
        self.used_counter == 0
    }

    fn destroy(self, device: &DeviceContext) {
        if self.used_counter != 0 {
            log::warn!("Destroying static image despite used counter being {:?}", self.used_counter);
        }

        unsafe {
            device.vk().destroy_image_view(self.view, None);
            device.get_functions().vk.destroy_image(self.image, None);
        }

        device.get_allocator().free(self.allocation);
    }

    fn create_image(device: &DeviceContext, size: Vec2u32, mip_levels: u32, format: vk::Format) -> (vk::Image, Allocation, vk::ImageView) {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: size[0],
                height: size[1],
                depth: 1
            })
            .mip_levels(mip_levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            device.vk().create_image(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateImage returned {:?} in StaticImage::create_image", err);
            panic!();
        });

        let allocation = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).unwrap_or_else(|err| {
            log::error!("allocate_image_memory failed with {:?} in StaticImage::create_image", err);
            panic!()
        });

        unsafe {
            device.vk().bind_image_memory(image, allocation.memory(), allocation.offset())
        }.unwrap_or_else(|err| {
            log::error!("vkBindImageMemory returned {:?} in StaticImage::create_image", err);
            panic!()
        });

        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1
            });

        let view = unsafe {
            device.vk().create_image_view(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateImageView returned {:?} in StaticImage::create_image", err);
            panic!()
        });

        (image, allocation, view)
    }
}