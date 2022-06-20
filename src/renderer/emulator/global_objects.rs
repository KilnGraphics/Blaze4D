use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use ash::vk;
use winit::event::VirtualKeyCode::M;
use crate::define_uuid_type;

use crate::device::device::Queue;
use crate::device::transfer::{BufferReleaseOp, BufferTransferRanges, SyncId};
use crate::objects::sync::{Semaphore, SemaphoreOp, SemaphoreOps};
use crate::renderer::emulator::MeshData;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::renderer::emulator::staging::{StagingAllocationId, StagingMemoryPool};
use crate::util::alloc::next_aligned;

/// Manages objects which are global to all passes of a emulator renderer.
///
/// This includes things like static meshes or static textures.
pub(super) struct GlobalObjects {
    queue_family: u32,
    data: Mutex<Data>,
}

impl GlobalObjects {
    /// Creates a new instance.
    ///
    /// The passed queue is the queue used for rendering. All created objects will be transferred to
    /// this queue family when accessed for rendering.
    pub(super) fn new(device: Arc<DeviceContext>, queue: Arc<Queue>) -> Self {
        let queue_family= queue.get_queue_family_index();
        let data = Data::new(device, queue);

        Self {
            queue_family,
            data: Mutex::new(data),
        }
    }

    /// Should be called regularly by the worker thread.
    ///
    /// This runs more heavy weight operations which have been deferred.
    ///
    /// Not calling this function will **never** cause blocked state. However it might cause
    /// inefficient performance or resource usage for example by not destroying unused objects.
    pub(super) fn update(&self) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned data mutex in GlobalObjects::update");
            panic!()
        }).update();
    }

    pub(super) fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::create_static_mesh!");
            panic!();
        }).create_static_mesh(data)
    }

    pub(super) fn mark_static_mesh(&self, id: StaticMeshId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::mark_static_mesh!");
            panic!();
        }).mark_static_mesh(id)
    }

    pub(super) fn inc_static_mesh(&self, id: StaticMeshId) -> StaticMeshDrawInfo {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::inc_static_mesh!");
            panic!();
        }).inc_get_static_mesh(id)
    }

    pub(super) fn dec_static_mesh(&self, id: StaticMeshId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::dec_static_mesh!");
            panic!();
        }).dec_static_mesh(id)
    }

    pub(super) fn create_static_image(&self, data: &StaticImageData) -> StaticImageId {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::create_static_image!");
            panic!();
        }).create_static_image(data)
    }

    pub(super) fn mark_static_image(&self, id: StaticImageId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::mark_static_image!");
            panic!();
        }).mark_static_image(id)
    }

    pub(super) fn inc_static_image(&self, id: StaticImageId) -> vk::ImageView {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::inc_static_image!");
            panic!();
        }).inc_get_static_image(id)
    }

    pub(super) fn dec_static_image(&self, id: StaticImageId) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::dec_static_image!");
            panic!();
        }).dec_static_image(id)
    }

    /// Flushes any pending operations which need to be executed on global objects.
    ///
    /// Calling this function ensures that all objects created or manipulated before this function
    /// is called are ready to be used by a pass. If [`Some`] is returned any caller must
    /// additionally wait on the semaphore before using any global object.
    ///
    /// This is a heavyweight operation and should ideally only be called from the worker thread.
    pub(super) fn flush(&self) {
        self.data.lock().unwrap_or_else(|_| {
            log::error!("Poisoned mutex in GlobalObjects::flush!");
            panic!();
        }).flush()
    }
}

struct Data {
    device: Arc<DeviceContext>,
    queue: Arc<Queue>,

    semaphore: Semaphore,
    semaphore_current_value: u64,

    staging_pool: StagingMemoryPool,
    command_pool: vk::CommandPool,
    available_command_buffers: Vec<vk::CommandBuffer>,
    pending_command_buffer: Option<vk::CommandBuffer>,
    submitted_command_buffers: VecDeque<(u64, vk::CommandBuffer, Vec<StagingAllocationId>)>,

    pending_buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
    pending_image_barriers: Vec<vk::ImageMemoryBarrier2>,
    pending_staging_allocations: Vec<StagingAllocationId>,

    static_meshes: HashMap<StaticMeshId, StaticMesh>,
    static_images: HashMap<StaticImageId, StaticImage>,
    droppable_static_meshes: Vec<StaticMesh>,
    droppable_static_images: Vec<StaticImage>,
}

impl Data {
    fn new(device: Arc<DeviceContext>, queue: Arc<Queue>) -> Self {
        let semaphore = Self::create_semaphore(device.get_functions());
        let command_pool = Self::create_command_pool(device.get_functions(), queue.get_queue_family_index());
        let staging_pool = StagingMemoryPool::new(device.clone());

        Self {
            device,
            queue,

            semaphore: Semaphore::new(semaphore),
            semaphore_current_value: 0,

            staging_pool,
            command_pool,
            available_command_buffers: Vec::new(),
            pending_command_buffer: None,
            submitted_command_buffers: VecDeque::new(),

            pending_buffer_barriers: Vec::new(),
            pending_image_barriers: Vec::new(),
            pending_staging_allocations: Vec::new(),

            static_meshes: HashMap::new(),
            static_images: HashMap::new(),
            droppable_static_meshes: Vec::new(),
            droppable_static_images: Vec::new(),
        }
    }

    fn create_static_mesh(&mut self, data: &MeshData) -> StaticMeshId {
        let index_offset = next_aligned(data.vertex_data.len() as vk::DeviceSize, data.get_index_size() as vk::DeviceSize);
        let required_size = index_offset + (data.index_data.len() as vk::DeviceSize);

        let (buffer, allocation) = StaticMesh::create_buffer(&self.device, required_size as usize);

        let (mapped, staging) = if let Some(mapped) = allocation.mapped_ptr() {
            (mapped.cast(), None)
        } else {
            let staging = self.staging_pool.allocate(required_size, 1);
            (staging.0.mapped, Some(staging))
        };

        unsafe {
            let dst = std::slice::from_raw_parts_mut(mapped.as_ptr(), required_size as usize);

            dst[0..data.vertex_data.len()].copy_from_slice(data.vertex_data);
            dst[(index_offset as usize)..].copy_from_slice(data.index_data);
        }

        if let Some((staging_alloc, staging_id)) = staging {
            let cmd = self.get_begin_pending_command_buffer();

            let region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: required_size as vk::DeviceSize,
            };

            unsafe {
                self.device.vk().cmd_copy_buffer(
                    cmd,
                    staging_alloc.buffer,
                    buffer.get_handle(),
                    std::slice::from_ref(&region)
                );
            }

            self.pending_buffer_barriers.push(vk::BufferMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT)
                .dst_access_mask(vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ)
                .buffer(buffer.get_handle())
                .offset(0)
                .size(required_size)
                .build()
            );

            self.pending_staging_allocations.push(staging_id);
        }

        let draw_info = StaticMeshDrawInfo {
            buffer,
            first_index: (index_offset / (data.get_index_size() as vk::DeviceSize)) as u32,
            index_type: data.index_type,
            index_count: data.index_count,
            primitive_topology: data.primitive_topology
        };

        let static_mesh = StaticMesh {
            buffer,
            allocation,
            draw_info,
            used_counter: 0,
            marked: false
        };

        let mesh_id = StaticMeshId::new();
        if self.static_meshes.insert(mesh_id, static_mesh).is_some() {
            log::error!("UUID collision");
            panic!();
        }

        mesh_id
    }

    fn create_static_image(&mut self, data: &StaticImageData) -> StaticImageId {
        let (image, allocation, view) = StaticImage::create_image(&self.device, data.size, data.generate_mip_levels, data.format);

        let staging = self.staging_pool.allocate(data.data.len() as vk::DeviceSize, 1);
        let mapped = staging.0.mapped;

        unsafe {
            let dst = std::slice::from_raw_parts_mut(mapped.as_ptr(), data.data.len());
            dst.copy_from_slice(data.data);
        }

        let cmd = self.get_begin_pending_command_buffer();

        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D {
                x: 0,
                y: 0,
                z: 0
            },
            image_extent: vk::Extent3D {
                width: data.size[0],
                height: data.size[1],
                depth: 1
            }
        };

        unsafe {
            self.device.vk().cmd_copy_buffer_to_image(
                cmd,
                staging.0.buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&region)
            );
        }

        let mut last_size = data.size;
        for level in 1..data.generate_mip_levels {
            let current_size = Vec2u32::new(
                if last_size[0] > 1 { last_size[0] / 2 } else { 1 },
                if last_size[1] > 1 { last_size[1] / 2 } else { 1 },
            );

            let barrier = vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::BLIT)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: level - 1,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                });

            let info = vk::DependencyInfo::builder()
                .image_memory_barriers(std::slice::from_ref(&barrier));

            unsafe {
                self.device.synchronization_2_khr().cmd_pipeline_barrier2(cmd, &info);
            }
            
            let region = vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level - 1,
                    base_array_layer: 0,
                    layer_count: 1
                },
                src_offsets: [
                    vk::Offset3D {
                        x: 0,
                        y: 0,
                        z: 0
                    },
                    vk::Offset3D {
                        x: last_size[0] as i32,
                        y: last_size[1] as i32,
                        z: 1
                    }
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level,
                    base_array_layer: 0,
                    layer_count: 1
                },
                dst_offsets: [
                    vk::Offset3D {
                        x: 0,
                        y: 0,
                        z: 0
                    },
                    vk::Offset3D {
                        x: current_size[0] as i32,
                        y: current_size[1] as i32,
                        z: 1
                    }
                ]
            };

            unsafe {
                self.device.vk().cmd_blit_image(
                    cmd,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&region),
                    vk::Filter::LINEAR,
                );
            }

            last_size = current_size;
        }

        if data.generate_mip_levels > 1 {
            self.pending_image_barriers.push(vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::BLIT)
                .src_access_mask(vk::AccessFlags2::TRANSFER_READ)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
                .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: data.generate_mip_levels - 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build()
            );
            self.pending_image_barriers.push(vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::BLIT)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
                .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: data.generate_mip_levels - 1,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build()
            );
        } else {
            self.pending_image_barriers.push(vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
                .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build()
            );
        }

        let static_image = StaticImage {
            image,
            allocation,
            view,
            used_counter: 0,
            marked: false
        };

        let image_id = StaticImageId::new();
        if self.static_images.insert(image_id, static_image).is_some() {
            log::error!("UUID collision");
            panic!();
        }

        image_id
    }

    fn mark_static_mesh(&mut self, mesh_id: StaticMeshId) {
        let mut drop = false;
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            static_mesh.marked = true;
            if static_mesh.is_unused() {
                drop = true;
            }
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::mark_static_mesh", mesh_id);
            panic!()
        }

        if drop {
            let static_mesh = self.static_meshes.remove(&mesh_id).unwrap();
            self.droppable_static_meshes.push(static_mesh);
        }
    }

    fn inc_get_static_mesh(&mut self, mesh_id: StaticMeshId) -> StaticMeshDrawInfo {
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            if !static_mesh.inc() {
                log::error!("Inc was called on marked static mesh!");
                panic!();
            }

            static_mesh.draw_info.clone()
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::inc_get_static_mesh", mesh_id);
            panic!()
        }
    }

    fn dec_static_mesh(&mut self, mesh_id: StaticMeshId) {
        let mut drop = false;
        if let Some(static_mesh) = self.static_meshes.get_mut(&mesh_id) {
            if static_mesh.dec() {
                drop = true;
            }
        } else {
            log::error!("Failed to find mesh with id {:?} in Data::dec_static_mesh", mesh_id);
            panic!()
        }

        if drop {
            let static_mesh = self.static_meshes.remove(&mesh_id).unwrap();
            self.droppable_static_meshes.push(static_mesh);
        }
    }

    fn mark_static_image(&mut self, image_id: StaticImageId) {
        let mut drop = false;
        if let Some(static_image) = self.static_images.get_mut(&image_id) {
            static_image.marked = true;
            if static_image.is_unused() {
                drop = true;
            }
        } else {
            log::error!("Failed to find image with id {:?} in Data::mark_static_image", image_id);
            panic!()
        }

        if drop {
            let static_image = self.static_images.remove(&image_id).unwrap();
            self.droppable_static_images.push(static_image);
        }
    }

    fn inc_get_static_image(&mut self, image_id: StaticImageId) -> vk::ImageView {
        if let Some(static_image) = self.static_images.get_mut(&image_id) {
            if !static_image.inc() {
                log::error!("Inc was called on marked static image!");
                panic!();
            }

            static_image.view
        } else {
            log::error!("Failed to find image wiht id {:?} in Data::inc_get_static_image", image_id);
            panic!()
        }
    }

    fn dec_static_image(&mut self, image_id: StaticImageId) {
        let mut drop = false;
        if let Some(static_image) = self.static_images.get_mut(&image_id) {
            if static_image.dec() {
                drop = true;
            }
        } else {
            log::error!("Failed to find image with id {:?} in Data::dec_static_image", image_id);
            panic!()
        }
    }

    fn update(&mut self) {
        let current_value = unsafe {
            self.device.timeline_semaphore_khr().get_semaphore_counter_value(self.semaphore.get_handle())
        }.unwrap_or_else(|err| {
            log::error!("vkGetSemaphoreCounterValue returned {:?} in Data::update", err);
            panic!()
        });

        while let Some((value, cmd, staging)) = self.submitted_command_buffers.pop_front() {
            if current_value >= value {
                self.available_command_buffers.push(cmd);
                for alloc in staging {
                    self.staging_pool.free(alloc);
                }
            } else {
                self.submitted_command_buffers.push_front((value, cmd, staging));
                break;
            }
        }

        while let Some(static_mesh) = self.droppable_static_meshes.pop() {
            static_mesh.destroy(&self.device);
        }
        while let Some(static_image) = self.droppable_static_images.pop() {
            static_image.destroy(&self.device);
        }
    }

    fn flush(&mut self) {
        let vk = self.device.vk();

        if let Some(cmd) = self.pending_command_buffer.take() {
            if !self.pending_buffer_barriers.is_empty() || !self.pending_image_barriers.is_empty() {
                let info = vk::DependencyInfo::builder()
                    .dependency_flags(vk::DependencyFlags::empty())
                    .buffer_memory_barriers(self.pending_buffer_barriers.as_slice())
                    .image_memory_barriers(self.pending_image_barriers.as_slice());

                unsafe {
                    self.device.synchronization_2_khr().cmd_pipeline_barrier2(cmd, &info);
                }

                self.pending_buffer_barriers.clear();
                self.pending_image_barriers.clear();
            }

            unsafe {
                vk.end_command_buffer(cmd)
            }.unwrap_or_else(|err| {
                log::error!("vkEndCommandBuffer returned {:?} in Data::flush!", err);
                panic!();
            });

            self.semaphore_current_value += 1;
            let signal_value = self.semaphore_current_value;

            let command_infos = [
                vk::CommandBufferSubmitInfo::builder()
                    .command_buffer(cmd)
                    .build(),
            ];

            let signal_infos = [
                vk::SemaphoreSubmitInfo::builder()
                    .semaphore(self.semaphore.get_handle())
                    .value(signal_value)
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                    .build(),
            ];

            let info = vk::SubmitInfo2::builder()
                .command_buffer_infos(&command_infos)
                .signal_semaphore_infos(&signal_infos);

            unsafe {
                self.queue.submit_2(std::slice::from_ref(&info), None)
            }.unwrap_or_else(|err| {
                log::error!("vkQueueSubmit2 returned {:?} in Data::flush!", err);
                panic!();
            });

            let staging_allocations = std::mem::replace(&mut self.pending_staging_allocations, Vec::new());
            self.submitted_command_buffers.push_back((signal_value, cmd, staging_allocations));
        }
    }

    fn get_begin_pending_command_buffer(&mut self) -> vk::CommandBuffer {
        if let Some(cmd) = self.pending_command_buffer {
            cmd
        } else {
            let cmd = self.get_begin_command_buffer();
            self.pending_command_buffer = Some(cmd);
            cmd
        }
    }

    fn get_begin_command_buffer(&mut self) -> vk::CommandBuffer {
        let cmd = self.get_command_buffer();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device.vk().begin_command_buffer(cmd, &info)
        }.unwrap_or_else(|err| {
            log::error!("vkBeginCommandBuffer returned {:?} in Data::get_begin_command_buffer!", err);
            panic!("");
        });

        cmd
    }

    fn get_command_buffer(&mut self) -> vk::CommandBuffer {
        if let Some(cmd) = self.available_command_buffers.pop() {
            return cmd;
        } else {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(4);

            let new_buffers = unsafe {
                self.device.vk().allocate_command_buffers(&info)
            }.unwrap_or_else(|err| {
                log::error!("vkAllocateCommandBuffers returned {:?} in Data::get_command_buffer", err);
                panic!();
            });

            self.available_command_buffers.extend(new_buffers);

            self.available_command_buffers.pop().unwrap()
        }
    }

    fn create_semaphore(device: &DeviceFunctions) -> vk::Semaphore {
        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        unsafe {
            device.vk.create_semaphore(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateSemaphore returned {:?} while trying to create GlobalObjects semaphore!", err);
            panic!()
        })
    }

    fn create_command_pool(device: &DeviceFunctions, queue_family: u32) -> vk::CommandPool {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family);

        unsafe {
            device.vk.create_command_pool(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateCommandPool returned {:?} in Data::create_command_pool!", err);
            panic!()
        })
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_semaphore(self.semaphore.get_handle(), None);
        }
    }
}

unsafe impl Send for Data { // Needed because of the pnext pointer in the memory barriers
}

define_uuid_type!(pub, StaticMeshId);

#[derive(Copy, Clone, Debug)]
pub struct StaticMeshDrawInfo {
    pub buffer: Buffer,
    pub first_index: u32,
    pub index_type: vk::IndexType,
    pub index_count: u32,
    pub primitive_topology: vk::PrimitiveTopology,
}

pub struct StaticMesh {
    buffer: Buffer,
    allocation: Allocation,
    draw_info: StaticMeshDrawInfo,

    used_counter: u32,
    marked: bool,
}

impl StaticMesh {
    /// Attempts to increment the used counter.
    ///
    /// If the mesh is marked the counter is not incremented and false is returned.
    fn inc(&mut self) -> bool {
        if self.marked {
            return false;
        }

        self.used_counter += 1;
        true
    }

    /// Decrements the used counter.
    ///
    /// If the mesh is marked and the counter decrements to 0 true is returned indicating that the
    /// mesh can be destroyed.
    fn dec(&mut self) -> bool {
        if self.used_counter == 0 {
            log::error!("Used counter is already 0 when calling StaticMesh::dec");
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
            log::warn!("Destroying static mesh despite used counter being {:?}", self.used_counter);
        }

        unsafe {
            device.get_functions().vk.destroy_buffer(self.buffer.get_handle(), None);
        }

        device.get_allocator().free(self.allocation);
    }

    fn create_buffer(device: &DeviceContext, size: usize) -> (Buffer, Allocation) {
        let vk = &device.get_functions().vk;

        let info = vk::BufferCreateInfo::builder()
            .size(size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            vk.create_buffer(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("vkCreateBuffer returned {:?} when trying to create buffer for static mesh of size {:?}", err, size);
            panic!()
        });

        let alloc = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuOnly)
            .unwrap_or_else(|err| {
                log::error!("allocate_buffer_memory failed with {:?} when trying to allocate memory for static mesh buffer of size {:?}", err, size);
                panic!()
            });

        unsafe {
            vk.bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
        }.unwrap_or_else(|err| {
            log::error!("vkBindBufferMemory returned {:?} when trying to bind memory for static mesh buffer of size {:?}", err, size);
            panic!()
        });

        (Buffer::new(buffer), alloc)
    }
}

define_uuid_type!(pub, StaticImageId);

pub struct StaticImageData<'a> {
    data: &'a [u8],
    format: vk::Format,
    size: Vec2u32,
    generate_mip_levels: u32,
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
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL);

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