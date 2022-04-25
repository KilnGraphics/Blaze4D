//! Debug overlay

use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Write;
use std::ops::Deref;
use std::sync::Mutex;
use ash::vk;
use crate::debug::text::{CharacterVertexData, FontData, TextColor, TextGenerator, TextSection, TextStyle};

use crate::prelude::*;
use crate::vk::device::VkQueue;
use crate::vk::DeviceContext;

use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub mod text;

pub struct DebugRenderer {
    elements: Mutex<HashMap<u64, DebugElement>>,
    device: DeviceContext,
    debug_vk: DebugVk,
}

impl DebugRenderer {
    pub fn new(device: DeviceContext) -> Self {
        let font = FontData::load();
        let data = [font.regular_image.0.as_ref()];

        let debug_vk = DebugVk::new(device.clone(), Vec2u32::new(256, 256), &data);

        let generator = TextGenerator::new(font.regular_style);
        let text = TextSection {
            text: "A TEEEEXT WHOWOWOW",
            color: TextColor {
                r: 255,
                g: 255,
                b: 255
            },
            style: TextStyle::Regular,
        };
        let (data, _) = generator.generate(std::slice::from_ref(&text));
        log::error!("Datga {:?}", data.as_ref());
        let data_bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * std::mem::size_of::<CharacterVertexData>()) };

        let mut data = vec![0u8; data_bytes.len()].into_boxed_slice();
        data.copy_from_slice(data_bytes);

        let element = DebugElement {
            position: Vec2f32::new(0.5, 0.5),
            size: Default::default(),
            text_data: data,
            refresh: false
        };

        let mut elements = HashMap::new();
        elements.insert(0, element);

        Self {
            elements: Mutex::new(elements),
            device,
            debug_vk
        }
    }

    pub fn draw_to_image(&self, image: vk::Image, view: vk::ImageView, draw_area: Vec2u32) {
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(self.debug_vk.render_pass)
            .attachments(std::slice::from_ref(&view))
            .width(draw_area[0])
            .height(draw_area[1])
            .layers(1);

        let framebuffer = unsafe { self.device.vk().create_framebuffer(&info, None) }.unwrap();

        self.run_draw_pass(&self.debug_vk, framebuffer, image, draw_area);

        unsafe { self.device.vk().destroy_framebuffer(framebuffer, None) };
    }

    fn run_draw_pass(&self, debug_vk: &DebugVk, framebuffer: vk::Framebuffer, image: vk::Image, draw_area: Vec2u32) {
        unsafe { *debug_vk.uniform_buffer_allocation.mapped_ptr().unwrap().cast::<Vec2f32>().as_mut() = draw_area.cast() };

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { self.device.vk().begin_command_buffer(debug_vk.command_buffer, &info) }.unwrap();

        let info = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        unsafe { self.device.vk().cmd_pipeline_barrier(
            debug_vk.command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&info)
        )};

        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1f32, 0f32, 0.1f32, 0f32]
            }
        };
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(debug_vk.render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D{ x: 0, y: 0 },
                extent: vk::Extent2D{ width: draw_area[0], height: draw_area[1] }
            })
            .clear_values(std::slice::from_ref(&clear_value));

        let viewport = vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(draw_area[0] as f32)
            .height(draw_area[1] as f32)
            .min_depth(0f32)
            .max_depth(1f32);

        unsafe { self.device.vk().cmd_set_viewport(debug_vk.command_buffer, 0, std::slice::from_ref(&viewport)) };

        unsafe { self.device.vk().cmd_begin_render_pass(debug_vk.command_buffer, &info, vk::SubpassContents::INLINE) };

        unsafe { self.device.vk().cmd_bind_pipeline(debug_vk.command_buffer, vk::PipelineBindPoint::GRAPHICS, debug_vk.text_pipeline) };
        unsafe { self.device.vk().cmd_bind_descriptor_sets(
            debug_vk.command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            debug_vk.text_layout,
            0,
            &[debug_vk.text_set],
            &[]
        )};

        unsafe { self.device.vk().cmd_bind_vertex_buffers(debug_vk.command_buffer, 0, &[debug_vk.vertex_buffer], &[0]) };

        let mapped = debug_vk.get_mapped();

        let mut mapped_offset = debug_vk.vertex_buffer_offset;

        for element in self.elements.lock().unwrap().values() {
            let draw_count = element.text_data.len() / std::mem::size_of::<CharacterVertexData>();

            let offset = unsafe { std::slice::from_raw_parts((&element.position).as_ptr() as *const u8, std::mem::size_of::<Vec2f32>()) };

            unsafe { self.device.vk().cmd_push_constants(debug_vk.command_buffer, debug_vk.text_layout, vk::ShaderStageFlags::VERTEX, 0, offset) };
            unsafe { self.device.vk().cmd_bind_vertex_buffers(debug_vk.command_buffer, 1, &[debug_vk.vertex_buffer], &[mapped_offset as u64]) };
            unsafe { self.device.vk().cmd_draw(debug_vk.command_buffer, 4, draw_count as u32, 0, 0) };

            mapped_offset += (&mut mapped[mapped_offset..]).write(element.text_data.as_ref()).unwrap();
        }

        unsafe { self.device.vk().cmd_end_render_pass(debug_vk.command_buffer) };
        unsafe { self.device.vk().end_command_buffer(debug_vk.command_buffer) }.unwrap();

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&debug_vk.command_buffer));

        unsafe { self.device.vk().reset_fences(std::slice::from_ref(&debug_vk.random_fence)) }.unwrap();

        unsafe { debug_vk.queue.submit(std::slice::from_ref(&submit_info), Some(debug_vk.random_fence)) }.unwrap();

        unsafe { self.device.vk().wait_for_fences(std::slice::from_ref(&debug_vk.random_fence), true, u64::MAX) }.unwrap();

        unsafe { self.device.vk().reset_command_pool(debug_vk.command_pool, vk::CommandPoolResetFlags::empty()) }.unwrap();
    }
}

struct DebugElement {
    position: Vec2f32,
    size: Vec2f32,
    text_data: Box<[u8]>,
    refresh: bool,
}

struct DebugVk {
    device: DeviceContext,
    queue: VkQueue,
    vertex_buffer: vk::Buffer,
    vertex_buffer_size: usize,
    vertex_buffer_allocation: Allocation,
    vertex_buffer_offset: usize,
    uniform_buffer: vk::Buffer,
    uniform_buffer_allocation: Allocation,
    atlas_image: vk::Image,
    atlas_image_size: Vec2u32,
    atlas_image_allocation: Allocation,
    atlas_image_views: Box<[vk::ImageView]>,
    sampler: vk::Sampler,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    render_pass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    text_set_layout: vk::DescriptorSetLayout,
    text_set: vk::DescriptorSet,
    text_layout: vk::PipelineLayout,
    text_pipeline: vk::Pipeline,
    random_fence: vk::Fence,
}

impl DebugVk {
    fn new(device: DeviceContext, atlas_image_size: Vec2u32, atlas_data: &[&[u8]]) -> Self {
        let atlas_count = atlas_data.len() as u32;

        let vertex_buffer_size = 32000000usize;
        let (vertex_buffer, vertex_buffer_allocation) = Self::create_vertex_buffer(&device, vertex_buffer_size as u64);
        let (uniform_buffer, uniform_buffer_allocation) = Self::create_uniform_buffer(&device);

        let (atlas_image, atlas_image_allocation, atlas_image_views) = Self::create_atlas_image(&device, &atlas_image_size, atlas_count);
        let sampler = Self::create_sampler(&device);

        let render_pass = Self::create_render_pass(&device);
        let (text_layout, text_set_layout) = Self::create_pipeline_layout(&device, atlas_count);
        let text_pipeline = Self::create_text_pipeline(&device, render_pass, text_layout);

        let (descriptor_pool, text_set) = Self::create_descriptor_set(&device, text_set_layout, uniform_buffer, sampler, atlas_image_views.as_ref());

        let queue = device.get_main_queue();
        let (command_pool, command_buffer) = Self::create_command_pool_buffer(&device, &queue);

        let mut mapped = unsafe { std::slice::from_raw_parts_mut(vertex_buffer_allocation.mapped_ptr().unwrap().as_ptr() as *mut u8, vertex_buffer_size) };
        Self::copy_atlas_images(atlas_data, mapped, atlas_image_size);
        let random_fence = Self::upload_atlas(&device, &queue, command_buffer, vertex_buffer, atlas_image, atlas_image_size, atlas_count);

        unsafe { device.vk().reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty() )}.unwrap();

        let vertex_multipliers = [Vec2f32::new(0f32, 0f32), Vec2f32::new(0f32, 1f32), Vec2f32::new(1f32, 0f32), Vec2f32::new(1f32, 1f32)];
        let byte_count = std::mem::size_of::<Vec2f32>() * vertex_multipliers.len();
        mapped.write(unsafe { std::slice::from_raw_parts(vertex_multipliers.as_ptr() as *const u8, byte_count) } ).unwrap();

        Self {
            device,
            queue,
            vertex_buffer,
            vertex_buffer_size,
            vertex_buffer_allocation,
            vertex_buffer_offset: byte_count,
            uniform_buffer,
            uniform_buffer_allocation,
            atlas_image,
            atlas_image_size,
            atlas_image_allocation,
            atlas_image_views,
            sampler,
            command_pool,
            command_buffer,
            render_pass,
            descriptor_pool,
            text_set_layout,
            text_set,
            text_layout,
            text_pipeline,
            random_fence
        }
    }

    // Technically unsafe
    pub fn get_mapped(&self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.vertex_buffer_allocation.mapped_ptr().unwrap().as_ptr() as *mut u8, self.vertex_buffer_size) }
    }

    fn create_vertex_buffer(device: &DeviceContext, size: u64) -> (vk::Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.vk().create_buffer(&info, None) }.unwrap();

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap();

        unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        }.unwrap();

        (buffer, allocation)
    }

    fn create_uniform_buffer(device: &DeviceContext) -> (vk::Buffer, Allocation) {
        let info = vk::BufferCreateInfo::builder()
            .size(8)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.vk().create_buffer(&info, None) }.unwrap();

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap();

        unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        }.unwrap();

        (buffer, allocation)
    }

    fn create_atlas_image(device: &DeviceContext, size: &Vec2u32, layers: u32) -> (vk::Image, Allocation, Box<[vk::ImageView]>) {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .extent(vk::Extent3D{
                width: size[0],
                height: size[1],
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(layers)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe { device.vk().create_image(&info, None) }.unwrap();

        let allocation = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).unwrap();

        unsafe {
            device.vk().bind_image_memory(image, allocation.memory(), allocation.offset())
        }.unwrap();

        let mut views = Vec::with_capacity(layers as usize);
        for i in 0..layers {
            let info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_SRGB)
                .components(vk::ComponentMapping{
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: i,
                    layer_count: 1
                });

            views.push(unsafe { device.vk().create_image_view(&info, None) }.unwrap());
        }

        (image, allocation, views.into_boxed_slice())
    }

    fn create_sampler(device: &DeviceContext) -> vk::Sampler {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(0f32)
            .anisotropy_enable(false)
            .compare_enable(false);

        let sampler = unsafe { device.vk().create_sampler(&info, None) }.unwrap();

        sampler
    }

    fn create_command_pool_buffer(device: &DeviceContext, queue: &VkQueue) -> (vk::CommandPool, vk::CommandBuffer) {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue.get_queue_family_index());

        let pool = unsafe { device.vk().create_command_pool(&info, None) }.unwrap();

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let buffer = *unsafe { device.vk().allocate_command_buffers(&info) }.unwrap().get(0).unwrap();

        (pool, buffer)
    }

    fn create_render_pass(device: &DeviceContext) -> vk::RenderPass {
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(vk::Format::B8G8R8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
        ];
        let color_attachements = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];
        let subpasses = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachements)
                .build(),
        ];

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses);

        let render_pass = unsafe { device.vk().create_render_pass(&info, None) }.unwrap();

        render_pass
    }

    fn create_pipeline_layout(device: &DeviceContext, atlas_count: u32) -> (vk::PipelineLayout, vk::DescriptorSetLayout) {
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(2)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(atlas_count)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings);

        let set_layout = unsafe { device.vk().create_descriptor_set_layout(&info, None) }.unwrap();

        let set_layouts = [set_layout];
        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&[vk::PushConstantRange{
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: 8
            }]);

        let pipeline_layout = unsafe { device.vk().create_pipeline_layout(&info, None) }.unwrap();

        (pipeline_layout, set_layout)
    }

    fn create_descriptor_set(device: &DeviceContext, set_layout: vk::DescriptorSetLayout, uniform_buffer: vk::Buffer, sampler: vk::Sampler, image_views: &[vk::ImageView]) -> (vk::DescriptorPool, vk::DescriptorSet) {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: 1
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 1
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1
            }
        ];
        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        let pool = unsafe { device.vk().create_descriptor_pool(&info, None) }.unwrap();

        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(std::slice::from_ref(&set_layout));

        let set = *unsafe { device.vk().allocate_descriptor_sets(&info) }.unwrap().get(0).unwrap();

        let uniform_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer)
            .offset(0)
            .range(8);

        let sampler_info = vk::DescriptorImageInfo::builder()
            .sampler(sampler);

        let mut image_infos = Vec::with_capacity(image_views.len());
        for view in image_views.iter() {
            image_infos.push(vk::DescriptorImageInfo {
                sampler: vk::Sampler::null(),
                image_view: *view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            });
        }

        let descriptor_writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&uniform_info))
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(std::slice::from_ref(&sampler_info))
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .image_info(image_infos.as_slice())
                .build(),
        ];
        unsafe { device.vk().update_descriptor_sets(&descriptor_writes, &[]) };

        (pool, set)
    }

    fn load_text_shaders(device: &DeviceContext) -> (vk::ShaderModule, vk::ShaderModule) {
        assert_eq!(TEXT_VERTEX_SHADER.len() % 4, 0);
        assert_eq!(TEXT_FRAGMENT_SHADER.len() % 4, 0);

        let vert_code = unsafe { std::slice::from_raw_parts(
            TEXT_VERTEX_SHADER.as_ptr() as *const u32,
            TEXT_VERTEX_SHADER.len() / 4
        )};
        let frag_code = unsafe { std::slice::from_raw_parts(
            TEXT_FRAGMENT_SHADER.as_ptr() as *const u32,
            TEXT_FRAGMENT_SHADER.len() / 4
        )};

        let info = vk::ShaderModuleCreateInfo::builder()
            .code(vert_code);

        let vertex_shader = unsafe { device.vk().create_shader_module(&info, None) }.unwrap();

        let info = vk::ShaderModuleCreateInfo::builder()
            .code(frag_code);

        let fragment_shader = unsafe { device.vk().create_shader_module(&info, None) }.unwrap();

        (vertex_shader, fragment_shader)
    }

    fn create_text_pipeline(device: &DeviceContext, render_pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) -> vk::Pipeline {
        let (vertex_shader, fragment_shader) = Self::load_text_shaders(device);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build()
        ];

        let input_state_bindings = vec![
            vk::VertexInputBindingDescription::builder()
                .binding(0)
                .input_rate(vk::VertexInputRate::VERTEX)
                .stride(std::mem::size_of::<Vec2f32>() as u32)
                .build(),
            vk::VertexInputBindingDescription::builder()
                .binding(1)
                .input_rate(vk::VertexInputRate::INSTANCE)
                .stride(std::mem::size_of::<CharacterVertexData>() as u32)
                .build()
        ];
        let input_state_attributes = vec![
            vk::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(8u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(2)
                .binding(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(16u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(3)
                .binding(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(24u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(4)
                .binding(1)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(32u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(5)
                .binding(1)
                .format(vk::Format::R8_UINT)
                .offset(35u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(7)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0)
                .build()
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(input_state_bindings.as_slice())
            .vertex_attribute_descriptions(input_state_attributes.as_slice());

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(false);

        let scissor = std::slice::from_ref(&vk::Rect2D{
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk::Extent2D{ width: 800u32, height: 600u32 }
        });
        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissors(scissor);

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

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

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

        let dynamic_state_state = std::slice::from_ref(&vk::DynamicState::VIEWPORT);
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_state_state);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            //.depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = *unsafe { device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None) }.unwrap().get(0).unwrap();

        unsafe { device.vk().destroy_shader_module(vertex_shader, None) };
        unsafe { device.vk().destroy_shader_module(fragment_shader, None) };

        pipeline
    }

    fn copy_atlas_images(src: &[&[u8]], dst: &mut [u8], size: Vec2u32) {
        let image_size = (size[0] * size[1] * 4) as usize;
        for (i, image) in src.iter().enumerate() {
            let offset = image_size * i;
            Self::copy_rgb_to_rgba(*image, &mut dst[offset..image_size])
        }
    }

    fn copy_rgb_to_rgba(src: &[u8], dst: &mut [u8]) {
        assert_eq!(src.len() % 3, 0);
        let end = src.len() / 3;
        for i in 0..end {
            let src_base = i * 3;
            let dst_base = i * 4;

            dst[dst_base + 0] = src[src_base + 0];
            dst[dst_base + 1] = src[src_base + 1];
            dst[dst_base + 2] = src[src_base + 2];
            dst[dst_base + 3] = 255u8;
        }
    }

    fn upload_atlas(device: &DeviceContext, queue: &VkQueue, cmd: vk::CommandBuffer, buffer: vk::Buffer, image: vk::Image, size: Vec2u32, count: u32) -> vk::Fence {
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { device.vk().begin_command_buffer(cmd, &info) }.unwrap();

        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: vk::REMAINING_MIP_LEVELS,
                base_array_layer: 0,
                layer_count: vk::REMAINING_ARRAY_LAYERS
            });

        unsafe { device.vk().cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier)
        )};

        let copy = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: count
            })
            .image_offset(vk::Offset3D {
                x: 0,
                y: 0,
                z: 0
            })
            .image_extent(vk::Extent3D {
                width: size[0],
                height: size[1],
                depth: 1
            });
        unsafe { device.vk().cmd_copy_buffer_to_image(cmd, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, std::slice::from_ref(&copy)) }

        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::NONE)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: vk::REMAINING_MIP_LEVELS,
                base_array_layer: 0,
                layer_count: vk::REMAINING_ARRAY_LAYERS
            });

        unsafe { device.vk().cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier)
        )};

        unsafe { device.vk().end_command_buffer(cmd) }.unwrap();

        let info = vk::FenceCreateInfo::builder();
        let fence = unsafe { device.vk().create_fence(&info, None) }.unwrap();

        let info = vk::SubmitInfo::builder()
            .command_buffers(std::slice::from_ref(&cmd));

        unsafe { queue.submit(std::slice::from_ref(&info), Some(fence)) }.unwrap();

        unsafe { device.vk().wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) }.unwrap();

        fence
    }
}

impl Drop for DebugVk {
    fn drop(&mut self) {
    }
}

const TEXT_VERTEX_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/vert.spv"));
const TEXT_FRAGMENT_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/frag.spv"));