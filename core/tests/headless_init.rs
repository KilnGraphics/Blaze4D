use std::ffi::{CStr, CString};
use vk_profiles_rs::vp;
use ash::vk;

use b4d_core::init::instance::*;
use b4d_core::rosella::VulkanVersion;
use b4d_core::util::debug_messenger::RustLogDebugMessenger;

mod test_common;

#[test]
fn init_no_feature() {
    env_logger::init();

    let profile = vp::LunargDesktopPortability2021::profile_properties();

    let mut config = InstanceCreateConfig::new(
        profile,
        VulkanVersion::VK_1_1,
        CString::from(CStr::from_bytes_with_nul(b"B4D_Test\0").unwrap()),
        vk::make_api_version(0, 0, 1, 0)
    );
    config.add_debug_messenger(Box::new(RustLogDebugMessenger::new()));

    let instance = create_instance(config).unwrap();
}