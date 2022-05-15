use std::ffi::CStr;
use std::sync::{Arc, Mutex};
use ash::vk;

use crate::prelude::*;
use crate::device::device::VkQueue;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

use super::manager::DebugOverlayImpl;
use super::manager::ResourceResetError;

#[derive(Clone)]
pub struct Target(Arc<Mutex<TargetShare>>, Arc<DebugOverlayImpl>);

impl Target {
    pub(super) fn new(share: Arc<Mutex<TargetShare>>, overlay: Arc<DebugOverlayImpl>) -> Self {
        Self(share, overlay)
    }

    /// Sets the size of the target image in pixels.
    ///
    /// Any old targets or cache will be dropped synchronously.
    /// Creation of any new resources will be delayed until they are needed.
    pub fn resize(&self, new_size: Vec2u32) {
        let device = self.1.get_device();

        let mut guard = self.0.lock().unwrap();
        guard.drop_targets(device);
        guard.drop_cache(device);
        guard.target_size = new_size;
    }

    /// Generates targets to apply the overlay to.
    ///
    /// This will run synchronously and may take a long time.
    ///
    /// Any previously created targets will be dropped.
    pub fn prepare_targets(&self, targets: &[ApplyTarget]) -> Result<(), PrepareTargetsError> {
        let device = self.1.get_device();

        let mut guard = self.0.lock().unwrap();
        if guard.cache.is_none() {
            guard.create_cache(device)?;
        }
        // Make sure any old targets have been destroyed
        guard.drop_targets(device);
        guard.create_targets(device, targets, self.1.get_target_draw_globals())?;

        Ok(())
    }

    /// Drops all currently stored targets and associated resources. If there are pending operations
    /// on the targets this function will block.
    pub fn drop_targets(&self) {
        let device = self.1.get_device();

        let mut guard = self.0.lock().unwrap();
        guard.drop_targets(device);
    }

    /// Uses the currently cached overlay data and applies it to a target.
    pub fn apply_overlay(&self, target: usize, wait_semaphore: Option<vk::Semaphore>, signal_semaphore: Option<vk::Semaphore>) -> Result<(), ApplyOverlayError> {
        let device = self.1.get_device();

        let mut guard = self.0.lock().unwrap();
        guard.apply_overlay(device, target, wait_semaphore, signal_semaphore)
    }
}

#[derive(Debug)]
pub enum PrepareTargetsError {
    ResourceReset,
    DeviceLost,
}

impl From<ResourceResetError> for PrepareTargetsError {
    fn from(err: ResourceResetError) -> Self {
        match err {
            ResourceResetError::OutOfHostMemory => Self::ResourceReset,
            ResourceResetError::OutOfDeviceMemory => Self::ResourceReset,
            ResourceResetError::DeviceLost => Self::DeviceLost,
        }
    }
}

pub struct ApplyTarget {
    pub image: vk::Image,
    pub format: vk::Format,
    pub subresource_range: vk::ImageSubresourceRange,
    pub src_layout: vk::ImageLayout,
    pub dst_layout: vk::ImageLayout,
}

pub enum ApplyOverlayError {
    /// Indicates that no valid targets exist.
    NoTargets,

    /// Indicates that valid targets exist but the specified target does not.
    InvalidTarget,

    /// Indicates that a VK_ERROR_OUT_OF_HOST_MEMORY has occurred and the overlay engine has freed
    /// all target resources as a result. The target will be in the inactive state.
    ResourceResetHostMemory,

    /// Indicates that a VK_ERROR_OUT_OF_DEVICE_MEMORY has occurred and the overlay engine has freed
    /// all target resources as a result. The target will be in the inactive state.
    ResourceResetDeviceMemory,

    /// Indicates that a VK_ERROR_DEVICE_LOST has occurred and the overlay engine has freed
    /// all target resources as a result. The target will be in the inactive state.
    ResourceResetDeviceLost,
}

pub(super) struct TargetShare {
    target_size: Vec2u32,
    cache: Option<TargetShareCache>,
    resources: Option<TargetShareResources>,
}

impl TargetShare {
    pub fn new(initial_size: Vec2u32) -> Self {
        Self {
            target_size: initial_size,
            cache: None,
            resources: None,
        }
    }

    pub fn create_cache(&mut self, device: &DeviceEnvironment) -> Result<(), ResourceResetError> {
        if self.cache.is_some() {
            panic!("Cache must be none when creating it");
        }
        self.cache = Some(unsafe { TargetShareCache::new(device, self.target_size) }?);

        Ok(())
    }

    pub fn create_targets(&mut self, device: &DeviceEnvironment, targets: &[ApplyTarget], globals: &TargetDrawGlobals) -> Result<(), PrepareTargetsError> {
        if self.resources.is_some() {
            panic!("Targets must be none when creating them");
        }
        let cache = self.cache.as_ref().expect("Cache must exist when creating targets");
        self.resources = Some(unsafe { TargetShareResources::new(device, cache, globals, targets) }.map_err(|err| {
            self.drop_cache(device);
            err
        })?);

        Ok(())
    }

    pub fn drop_cache(&mut self, device: &DeviceEnvironment) {
        if self.resources.is_some() {
            panic!("Targets must be dropped before the cache may be dropped");
        }

        if let Some(mut cache) = self.cache.take() {
            cache.destroy(device);
        }
    }

    pub fn drop_targets(&mut self, device: &DeviceEnvironment) {
        if let Some(resources) = self.resources.take() {
            resources.destroy(device);
        }
    }

    pub fn apply_overlay(&mut self, device: &DeviceEnvironment, target: usize, wait_semaphore: Option<vk::Semaphore>, signal_semaphore: Option<vk::Semaphore>) -> Result<(), ApplyOverlayError> {
        let result = self.apply_overlay_impl(device, target, wait_semaphore, signal_semaphore);
        if let Err(err) = &result {
            match err {
                ApplyOverlayError::ResourceResetHostMemory |
                ApplyOverlayError::ResourceResetDeviceMemory |
                ApplyOverlayError::ResourceResetDeviceLost => {
                    self.drop_targets(device);
                    self.drop_cache(device);
                }
                _ => {}
            }
        }
        result
    }

    fn apply_overlay_impl(&mut self, device: &DeviceEnvironment, target: usize, wait_semaphore: Option<vk::Semaphore>, signal_semaphore: Option<vk::Semaphore>) -> Result<(), ApplyOverlayError> {
        if let Some(resources) = &self.resources {
            let buffer = resources.command_buffers.get(target).ok_or(ApplyOverlayError::InvalidTarget)?;

            let mut cache = self.cache.as_mut().expect("Target share cache must exist if targets exist");

            let mut wait_semaphores = Vec::with_capacity(2);
            let mut wait_stage_mask = Vec::with_capacity(2);
            if let Some(semaphore) = wait_semaphore {
                wait_semaphores.push(semaphore);
                wait_stage_mask.push(vk::PipelineStageFlags::FRAGMENT_SHADER);
            }
            if cache.apply_should_wait {
                wait_semaphores.push(cache.apply_wait_semaphore);
                wait_stage_mask.push(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT);
            }

            let mut signal_semaphores = Vec::with_capacity(2);
            if let Some(semaphore) = signal_semaphore {
                signal_semaphores.push(semaphore);
            }
            //signal_semaphores.push(cache.draw_wait_semaphore);

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stage_mask)
                .command_buffers(std::slice::from_ref(buffer))
                .signal_semaphores(&signal_semaphores);

            let result = unsafe { device.vk().wait_for_fences(std::slice::from_ref(&resources.apply_wait_fence), true, u64::MAX) };
            if let Err(result) = result {
                log::error!("Failed to wait for fences {:?}", result);
                return Err(match result {
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY => ApplyOverlayError::ResourceResetHostMemory,
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => ApplyOverlayError::ResourceResetDeviceMemory,
                    vk::Result::ERROR_DEVICE_LOST => ApplyOverlayError::ResourceResetDeviceLost,
                    _ => panic!("Unexpected result while waiting for fences: {:?}", result)
                });
            }
            let result = unsafe { device.vk().reset_fences(std::slice::from_ref(&resources.apply_wait_fence)) };
            if let Err(result) = result {
                log::error!("Failed to reset fences {:?}", result);
                return Err(match result {
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => ApplyOverlayError::ResourceResetDeviceMemory,
                    _ => panic!("Unexpected result while resetting fences: {:?}", result)
                });
            }

            let result = unsafe { resources.queue.submit(std::slice::from_ref(&submit_info), Some(resources.apply_wait_fence)) };
            if let Err(result) = result {
                log::error!("Failed to submit to queue {:?}", result);
                return Err(match result {
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY => ApplyOverlayError::ResourceResetHostMemory,
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => ApplyOverlayError::ResourceResetDeviceMemory,
                    vk::Result::ERROR_DEVICE_LOST => ApplyOverlayError::ResourceResetDeviceLost,
                    _ => panic!("Unexpected result while submitting to queue: {:?}", result)
                });
            }

            // Update cache information
            cache.apply_should_wait = false;
            cache.draw_should_wait = true;

            Ok(())
        } else {
            Err(ApplyOverlayError::NoTargets)
        }
    }
}

struct TargetShareCache {
    apply_wait_semaphore: vk::Semaphore,
    draw_wait_semaphore: vk::Semaphore,
    draw_wait_fence: vk::Fence,
    current_size: Vec2u32,
    image: vk::Image,
    image_allocation: Option<Allocation>,
    view: vk::ImageView,
    apply_should_wait: bool,
    draw_should_wait: bool,
}

impl TargetShareCache {
    pub const CACHE_IMAGE_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;

    unsafe fn new(device: &DeviceEnvironment, size: Vec2u32) -> Result<Self, ResourceResetError> {
        let (image, allocation, view) = Self::create_image(device, size)?;

        let stuff = Self::create_semaphores_fence(device);
        let (apply_wait_semaphore, draw_wait_semaphore, draw_wait_fence) = match stuff {
            Ok(stuff) => stuff,
            Err(err) => {
                device.vk().destroy_image_view(view, None);
                device.vk().destroy_image(image, None);
                device.get_allocator().free(allocation);
                return Err(err);
            }
        };

        Ok(Self {
            apply_wait_semaphore,
            draw_wait_semaphore,
            draw_wait_fence,
            current_size: size,
            image,
            image_allocation: Some(allocation),
            view,
            apply_should_wait: false,
            draw_should_wait: false
        })
    }

    unsafe fn create_image(device: &DeviceEnvironment, size: Vec2u32) -> Result<(vk::Image, Allocation, vk::ImageView), ResourceResetError> {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(Self::CACHE_IMAGE_FORMAT)
            .extent(vk::Extent3D {
                width: size[0],
                height: size[1],
                depth: 1
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = device.vk().create_image(&info, None).map_err(|err| {
            log::error!("Failed to create image: {:?}", err);
            err
        })?;

        let allocation = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).map_err(|err| {
            log::error!("Failed to allocate image memory: {:?}", err);
            device.vk().destroy_image(image, None);
            ResourceResetError::OutOfDeviceMemory
        })?;

        if let Err(result) = device.vk().bind_image_memory(image, allocation.memory(), allocation.offset()) {
            log::error!("Failed to bind image memory: {:?}", result);
            device.get_allocator().free(allocation);
            device.vk().destroy_image(image, None);
            return Err(ResourceResetError::from(result));
        };

        Self::sync_image_clear(device, image);

        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(Self::CACHE_IMAGE_FORMAT)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        let image_view = device.vk().create_image_view(&info, None);
        let image_view = match image_view {
            Ok(view) => view,
            Err(result) => {
                log::error!("Failed to create image view: {:?}", result);
                device.get_allocator().free(allocation);
                device.vk().destroy_image(image, None);
                return Err(ResourceResetError::from(result));
            }
        };

        Ok((image, allocation, image_view))
    }

    unsafe fn create_semaphores_fence(device: &DeviceEnvironment) -> Result<(vk::Semaphore, vk::Semaphore, vk::Fence), ResourceResetError> {
        let info = vk::SemaphoreCreateInfo::builder();

        let semaphore1 = device.vk().create_semaphore(&info, None)?;
        let semaphore2 = device.vk().create_semaphore(&info, None).map_err(|err| {
            unsafe { device.vk().destroy_semaphore(semaphore1, None) };
            err
        })?;

        let info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let fence = device.vk().create_fence(&info, None).map_err(|err| {
            device.vk().destroy_semaphore(semaphore1, None);
            device.vk().destroy_semaphore(semaphore2, None);
            err
        })?;

        Ok((semaphore1, semaphore2, fence))
    }

    fn destroy(&mut self, device: &DeviceEnvironment) {
        // If we cant wait we just have to continue destroying the resources and hope it will work
        if let Err(result) = unsafe { device.vk().wait_for_fences(std::slice::from_ref(&self.draw_wait_fence), true, u64::MAX) } {
            log::error!("Failed to wait for fences: {:?}", result)
        }

        unsafe { device.vk().destroy_fence(self.draw_wait_fence, None) };

        unsafe { device.vk().destroy_semaphore(self.apply_wait_semaphore, None) };
        unsafe { device.vk().destroy_semaphore(self.draw_wait_semaphore, None) };

        unsafe { device.vk().destroy_image_view(self.view, None) };
        unsafe { device.vk().destroy_image(self.image, None) };

        device.get_allocator().free(self.image_allocation.take().unwrap());
    }

    // **VERY** slow but necessary for now
    unsafe fn sync_image_clear(device: &DeviceEnvironment, image: vk::Image) {
        let queue = device.get_device().get_main_queue();

        let info = vk::FenceCreateInfo::builder();
        let fence = device.vk().create_fence(&info, None).unwrap();

        let info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue.get_queue_family_index());
        let command_pool = device.vk().create_command_pool(&info, None).unwrap();

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);
        let cmd = *device.vk().allocate_command_buffers(&info).unwrap().get(0).unwrap();

        let info = vk::CommandBufferBeginInfo::builder();
        device.vk().begin_command_buffer(cmd, &info).unwrap();

        let full_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1
        };

        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image)
            .subresource_range(full_range);

        device.vk().cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier)
        );

        let clear_color = vk::ClearColorValue {
            float32: [0f32, 0f32, 0f32, 0f32]
        };

        device.vk().cmd_clear_color_image(cmd, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &clear_color, std::slice::from_ref(&full_range));

        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image)
            .subresource_range(full_range);

        device.vk().cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier)
        );

        device.vk().end_command_buffer(cmd).unwrap();

        let info = vk::SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&cmd));

        queue.submit(std::slice::from_ref(&info), Some(fence)).unwrap();

        device.vk().wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX).unwrap();

        device.vk().destroy_command_pool(command_pool, None);
        device.vk().destroy_fence(fence, None);
    }
}

struct TargetShareResources {
    queue: VkQueue,
    apply_wait_fence: vk::Fence,
    descriptor_pool: vk::DescriptorPool,
    #[allow(unused)] // We only need this during command recording and its dropped together with the pool
    descriptor_set: vk::DescriptorSet,
    command_pool: vk::CommandPool,
    command_buffers: Box<[vk::CommandBuffer]>,
    framebuffers: Box<[vk::Framebuffer]>,
    target_views: Box<[vk::ImageView]>,
    pipelines: Box<[(vk::RenderPass, vk::Pipeline)]>,
}

impl TargetShareResources {
    unsafe fn new(device: &DeviceEnvironment, cache: &TargetShareCache, globals: &TargetDrawGlobals, targets: &[ApplyTarget]) -> Result<Self, PrepareTargetsError> {
        let target_count = targets.len() as u32;

        let (command_pool, command_buffers) = Self::create_command_buffers(device, globals, target_count)?;

        let (descriptor_pool, descriptor_set) = Self::create_descriptor_set(device, globals, cache.view).map_err(|err| {
            device.vk().destroy_command_pool(command_pool, None);
            err
        })?;

        let mut library = Vec::with_capacity(1);
        let mut target_views = Vec::with_capacity(targets.len());
        let mut framebuffers = Vec::with_capacity(targets.len());
        for (index, target) in targets.iter().enumerate() {
            let cmd = *command_buffers.get(index).unwrap();
            let result = Self::create_target(device, globals, cache, &mut library, target, descriptor_set, cmd);
            let (image_view, framebuffer) = match result {
                Ok(stuff) => stuff,
                Err(err) => {
                    device.vk().destroy_command_pool(command_pool, None);
                    for framebuffer in framebuffers.iter() { device.vk().destroy_framebuffer(*framebuffer, None); }
                    for image_view in target_views.iter() { device.vk().destroy_image_view(*image_view, None); }
                    device.vk().destroy_descriptor_pool(descriptor_pool, None);
                    for (_, render_pass, pipeline) in library {
                        device.vk().destroy_pipeline(pipeline, None);
                        device.vk().destroy_render_pass(render_pass, None);
                    }
                    return Err(err.into());
                }
            };

            target_views.push(image_view);
            framebuffers.push(framebuffer);
        }
        let target_views = target_views.into_boxed_slice();
        let framebuffers = framebuffers.into_boxed_slice();

        let result = Self::create_apply_wait_fence(device);
        let apply_wait_fence = match result {
            Ok(stuff) => stuff,
            Err(err) => {
                device.vk().destroy_command_pool(command_pool, None);
                for framebuffer in framebuffers.iter() { device.vk().destroy_framebuffer(*framebuffer, None); }
                for image_view in target_views.iter() { device.vk().destroy_image_view(*image_view, None); }
                device.vk().destroy_descriptor_pool(descriptor_pool, None);
                for (_, render_pass, pipeline) in library {
                    device.vk().destroy_pipeline(pipeline, None);
                    device.vk().destroy_render_pass(render_pass, None);
                }
                return Err(err.into());
            }
        };

        let pipelines: Box<_> = library.into_iter().map(|(_, render_pass, pipeline)| (render_pass, pipeline)).collect();

        Ok(Self {
            queue: globals.queue.clone(),
            apply_wait_fence,
            descriptor_pool,
            descriptor_set,
            command_pool,
            command_buffers,
            framebuffers,
            target_views,
            pipelines,
        })
    }

    unsafe fn create_command_buffers(device: &DeviceEnvironment, globals: &TargetDrawGlobals, target_count: u32) -> Result<(vk::CommandPool, Box<[vk::CommandBuffer]>), ResourceResetError> {
        let info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(globals.queue.get_queue_family_index());

        let result = device.vk().create_command_pool(&info, None);
        let command_pool = match result {
            Ok(pool) => pool,
            Err(result) => {
                log::error!("Failed to create command pool: {:?}", result);
                return Err(result.into());
            }
        };

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(target_count);

        let result = device.vk().allocate_command_buffers(&info);
        let command_buffers = match result {
            Ok(buffers) => buffers.into_boxed_slice(),
            Err(result) => {
                log::error!("Failed to allocate command buffers: {:?}", result);
                device.vk().destroy_command_pool(command_pool, None);
                return Err(result.into());
            }
        };

        Ok((command_pool, command_buffers))
    }

    unsafe fn create_descriptor_set(device: &DeviceEnvironment, globals: &TargetDrawGlobals, attachment_view: vk::ImageView)
                                    -> Result<(vk::DescriptorPool, vk::DescriptorSet), ResourceResetError>
    {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: 1
            }
        ];

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        let result = device.vk().create_descriptor_pool(&info, None);
        let descriptor_pool = match result {
            Ok(pool) => pool,
            Err(result) => {
                log::error!("Failed to create descriptor pool: {:?}", result);
                return Err(result.into());
            }
        };

        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&globals.descriptor_set_layout));

        let result = device.vk().allocate_descriptor_sets(&info);
        let descriptor_set = *match result {
            Ok(pool) => pool,
            Err(result) => {
                device.vk().destroy_descriptor_pool(descriptor_pool, None);
                return Err(result.into());
            }
        }.get(0).unwrap();

        let descriptor_image_info = vk::DescriptorImageInfo::builder()
            .image_view(attachment_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let info = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .image_info(std::slice::from_ref(&descriptor_image_info));

        device.vk().update_descriptor_sets(std::slice::from_ref(&info), &[]);

        Ok((descriptor_pool, descriptor_set))
    }

    unsafe fn create_target(
        device: &DeviceEnvironment,
        globals: &TargetDrawGlobals,
        cache: &TargetShareCache,
        library: &mut Vec<(vk::Format, vk::RenderPass, vk::Pipeline)>,
        target: &ApplyTarget,
        descriptor_set: vk::DescriptorSet,
        cmd: vk::CommandBuffer
    ) -> Result<(vk::ImageView, vk::Framebuffer), ResourceResetError> {
        let (render_pass, pipeline) = Self::get_or_create_pipeline(device, globals, target.format, library, cache.current_size)?;

        let info = vk::ImageViewCreateInfo::builder()
            .image(target.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(target.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(target.subresource_range);

        let result = device.vk().create_image_view(&info, None);
        let image_view = match result {
            Ok(view) => view,
            Err(result) => {
                log::error!("Failed to create image view: {:?}", result);
                return Err(result.into());
            }
        };

        let attachments = [image_view, cache.view];
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(cache.current_size[0])
            .height(cache.current_size[1])
            .layers(1);

        let result = device.vk().create_framebuffer(&info, None);
        let framebuffer = match result {
            Ok(framebuffer) => framebuffer,
            Err(result) => {
                log::error!("Failed to create framebuffer: {:?}", result);
                device.vk().destroy_image_view(image_view, None);
                return Err(result.into());
            }
        };

        Self::record_command_buffer(device, cmd, globals, cache, target, descriptor_set, framebuffer, render_pass, pipeline).map_err(|err| {
            device.vk().destroy_framebuffer(framebuffer, None);
            device.vk().destroy_image_view(image_view, None);
            err
        })?;

        Ok((image_view, framebuffer))
    }

    unsafe fn get_or_create_pipeline(device: &DeviceEnvironment, globals: &TargetDrawGlobals, format: vk::Format, library: &mut Vec<(vk::Format, vk::RenderPass, vk::Pipeline)>, size: Vec2u32) -> Result<(vk::RenderPass, vk::Pipeline), ResourceResetError> {
        for entry in library.iter() {
            if entry.0 == format {
                return Ok((entry.1, entry.2));
            }
        }

        let render_pass = Self::create_render_pass(device, format)?;
        let pipeline = Self::create_pipeline(device, globals, render_pass, size).map_err(|err| {
            device.vk().destroy_render_pass(render_pass, None);
            err
        })?;

        library.push((format, render_pass, pipeline));

        Ok((render_pass, pipeline))
    }

    unsafe fn create_render_pass(device: &DeviceEnvironment, format: vk::Format) -> Result<vk::RenderPass, ResourceResetError> {
        let attachments = [
            // This is the target to which we apply the overlay
            vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build(),
            // The input attachment containing the cached overlay
            vk::AttachmentDescription::builder()
                .format(TargetShareCache::CACHE_IMAGE_FORMAT)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::NONE)
                .initial_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build()
        ];
        let input_attachments = [
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            }
        ];
        let color_attachments = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];
        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .input_attachments(&input_attachments)
            .color_attachments(&color_attachments);

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass_description));

        let render_pass = device.vk().create_render_pass(&info, None)?;

        Ok(render_pass)
    }

    unsafe fn create_pipeline(device: &DeviceEnvironment, globals: &TargetDrawGlobals, render_pass: vk::RenderPass, size: Vec2u32) -> Result<vk::Pipeline, ResourceResetError> {
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(globals.vertex_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(globals.fragment_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: size[0] as f32,
            height: size[1] as f32,
            min_depth: 0.0,
            max_depth: 1.0
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D { width: size[0], height: size[1] }
        };
        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0f32);

        let multisample = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let color_blend_attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD)
                .build()
        ];
        let color_blend = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend)
            .layout(globals.pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = *device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
            .map_err(|(tmp, res)| {
                if !tmp.is_empty() {
                    panic!("How is this possible?")
                }
                res
            })?.get(0).unwrap();

        Ok(pipeline)
    }

    unsafe fn record_command_buffer(
        device: &DeviceEnvironment,
        cmd: vk::CommandBuffer,
        globals: &TargetDrawGlobals,
        cache: &TargetShareCache,
        target: &ApplyTarget,
        descriptor_set: vk::DescriptorSet,
        framebuffer: vk::Framebuffer,
        render_pass: vk::RenderPass,
        pipeline: vk::Pipeline
    ) -> Result<(), ResourceResetError> {
        let info = vk::CommandBufferBeginInfo::builder();

        let result = device.vk().begin_command_buffer(cmd, &info);
        if let Err(result) = result {
            log::error!("Failed to begin command buffer recording: {:?}", result);
            return Err(result.into());
        }

        if target.src_layout != vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
            let info = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .old_layout(target.src_layout)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image(target.image)
                .subresource_range(target.subresource_range);

            device.vk().cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&info)
            );
        }

        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D{ x: 0, y: 0 },
                extent: vk::Extent2D{ width: cache.current_size[0], height: cache.current_size[1] }
            });
        device.vk().cmd_begin_render_pass(cmd, &info, vk::SubpassContents::INLINE);
        device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);
        device.vk().cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            globals.pipeline_layout,
            0,
            std::slice::from_ref(&descriptor_set),
            &[]
        );
        device.vk().cmd_draw(cmd, 4, 1, 0, 0);
        device.vk().cmd_end_render_pass(cmd);

        if target.dst_layout != vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
            let info = vk::ImageMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(target.dst_layout)
                .image(target.image)
                .subresource_range(target.subresource_range);

            device.vk().cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&info)
            );
        }

        let result = device.vk().end_command_buffer(cmd);
        if let Err(result) = result {
            log::error!("Failed to end command buffer recording: {:?}", result);
            return Err(result.into());
        }

        Ok(())
    }

    unsafe fn create_apply_wait_fence(device: &DeviceEnvironment) -> Result<vk::Fence, ResourceResetError> {
        let info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        let result = device.vk().create_fence(&info, None);
        let fence = match result {
            Ok(fence) => fence,
            Err(result) => {
                log::error!("Failed to create fence: {:?}", result);
                return Err(result.into());
            }
        };

        Ok(fence)
    }

    fn destroy(&self, device: &DeviceEnvironment) {
        // If we cant wait we just have to continue destroying the resources and hope it will work
        if let Err(result) = unsafe { device.vk().wait_for_fences(std::slice::from_ref(&self.apply_wait_fence), true, u64::MAX) } {
            log::error!("Failed to wait for fences: {:?}", result);
        }

        unsafe { device.vk().destroy_fence(self.apply_wait_fence, None) };

        unsafe { device.vk().destroy_command_pool(self.command_pool, None) };

        unsafe { device.vk().destroy_descriptor_pool(self.descriptor_pool, None) };

        for framebuffer in self.framebuffers.iter() {
            unsafe { device.vk().destroy_framebuffer(*framebuffer, None) };
        }

        for view in self.target_views.iter() {
            unsafe { device.vk().destroy_image_view(*view, None) };
        }

        for (render_pass, pipeline) in self.pipelines.iter() {
            unsafe {
                device.vk().destroy_pipeline(*pipeline, None);
                device.vk().destroy_render_pass(*render_pass, None);
            }
        }
    }
}

pub(super) struct TargetDrawGlobals {
    queue: VkQueue,
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
}

impl TargetDrawGlobals {
    pub fn new(device: &DeviceEnvironment, queue: VkQueue) -> Result<Self, ResourceResetError> {
        let (vertex_shader, fragment_shader) = unsafe { Self::load_shader_modules(device) }?;

        let (descriptor_set_layout, pipeline_layout) = unsafe { Self::create_layouts(device) }.map_err(|err| {
            unsafe { device.vk().destroy_shader_module(vertex_shader, None) };
            unsafe { device.vk().destroy_shader_module(fragment_shader, None) };
            err
        })?;

        Ok(Self {
            queue,
            vertex_shader,
            fragment_shader,
            descriptor_set_layout,
            pipeline_layout
        })
    }

    unsafe fn load_shader_modules(device: &DeviceEnvironment) -> Result<(vk::ShaderModule, vk::ShaderModule), ResourceResetError> {
        let vert_code = std::slice::from_raw_parts(
            APPLY_VERTEX_SHADER.as_ptr() as *const u32,
            APPLY_VERTEX_SHADER.len() / 4
        );
        let frag_code = std::slice::from_raw_parts(
            APPLY_FRAGMENT_SHADER.as_ptr() as *const u32,
            APPLY_FRAGMENT_SHADER.len() / 4
        );

        let info = vk::ShaderModuleCreateInfo::builder()
            .code(vert_code);
        let vertex_shader = device.vk().create_shader_module(&info, None)?;

        let info = vk::ShaderModuleCreateInfo::builder()
            .code(frag_code);
        let fragment_shader = device.vk().create_shader_module(&info, None).map_err(|err| {
            device.vk().destroy_shader_module(vertex_shader, None);
            err
        })?;

        Ok((vertex_shader, fragment_shader))
    }

    unsafe fn create_layouts(device: &DeviceEnvironment) -> Result<(vk::DescriptorSetLayout, vk::PipelineLayout), ResourceResetError> {
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings);

        let set_layout = device.vk().create_descriptor_set_layout(&info, None)?;

        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&set_layout));

        let pipeline_layout = device.vk().create_pipeline_layout(&info, None).map_err(|err| {
            device.vk().destroy_descriptor_set_layout(set_layout, None);
            err
        })?;

        Ok((set_layout, pipeline_layout))
    }

    pub fn destroy(&mut self, device: &DeviceEnvironment) {
        unsafe {
            device.vk().destroy_shader_module(self.vertex_shader, None);
            device.vk().destroy_shader_module(self.fragment_shader, None);
            device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
            device.vk().destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

const APPLY_VERTEX_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/apply_vert.spv"));
const APPLY_FRAGMENT_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/apply_frag.spv"));

const_assert_eq!(APPLY_VERTEX_SHADER.len() % 4, 0);
const_assert_eq!(APPLY_FRAGMENT_SHADER.len() % 4, 0);