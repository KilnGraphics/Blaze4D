use std::ffi::{CStr, CString};
use vk_profiles_rs::vp;
use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use b4d_core::init::device::{create_device, DeviceCreateConfig};

use b4d_core::init::instance::*;
use b4d_core::objects::surface::SurfaceProvider;
use b4d_core::objects::swapchain::{SwapchainCreateDesc, SwapchainImageSpec};
use b4d_core::objects::{Format, SwapchainObjectSetBuilder};
use b4d_core::rosella::VulkanVersion;
use b4d_core::util::debug_messenger::RustLogDebugMessenger;
use b4d_core::window::WinitWindow;

mod test_common;

#[test]
fn init_no_feature() {
    env_logger::init();

    let event_loop: EventLoop<()> = winit::platform::unix::EventLoopExtUnix::new_any_thread();
    let surface: Box<dyn SurfaceProvider> = Box::new(WinitWindow::new("YASSSS", 800.0, 600.0, &event_loop));

    let profile = vp::LunargDesktopPortability2021::profile_properties();

    let mut config = InstanceCreateConfig::new(
        profile,
        VulkanVersion::VK_1_1,
        CString::from(CStr::from_bytes_with_nul(b"B4D_Test\0").unwrap()),
        vk::make_api_version(0, 0, 1, 0)
    );
    config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));
    config.request_min_api_version(VulkanVersion::VK_1_3);
    let surface = config.add_surface_provider(surface);

    let instance = create_instance(config).unwrap();


    let mut config = DeviceCreateConfig::new();
    config.add_surface(surface);

    let device = create_device(config, instance.clone()).unwrap();

    let swapchain_desc = SwapchainCreateDesc::make(
        SwapchainImageSpec::make(
            &Format::R8G8B8A8_SRGB,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
            800,
            600
        ),
        1,
        vk::ImageUsageFlags::COLOR_ATTACHMENT,
        vk::PresentModeKHR::FIFO
    );
    let semaphore = vk::SemaphoreCreateInfo::builder();
    let semaphore = unsafe { device.vk().create_semaphore(&semaphore, None) }.unwrap();

    let swapchain = SwapchainObjectSetBuilder::new(device.clone(), surface, swapchain_desc, None).unwrap();

    let swapchain_id = swapchain.get_swapchain_id();
    let images = swapchain.get_image_ids();

    let swapchain = swapchain.build().unwrap();
    let swapchain_handle = unsafe { swapchain.get_swapchain_handle(swapchain_id) };

    let khr = device.swapchain_khr().unwrap();
    let (index, _ ) = unsafe {
        khr.acquire_next_image(swapchain_handle, 0, semaphore, vk::Fence::null()).unwrap()
    };

    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(std::slice::from_ref(&semaphore))
        .swapchains(std::slice::from_ref(&swapchain_handle))
        .image_indices(std::slice::from_ref(&index));
    unsafe { device.get_surface_queue(surface).unwrap().present(&present_info) };

    event_loop.run(move |event, _, cfg| {
        *cfg = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *cfg = ControlFlow::Exit
            },
            _ => {
            }
        }
    });
}