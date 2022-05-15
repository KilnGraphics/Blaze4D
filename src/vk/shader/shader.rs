use ash::prelude::VkResult;
use ash::vk;

use crate::vk::DeviceEnvironment;

pub struct ShaderModule {
    device: DeviceEnvironment,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: DeviceEnvironment, code: &[u32]) -> VkResult<Self> {
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(code);

        let module = unsafe { device.vk().create_shader_module(&create_info, None) }?;

        Ok(Self {
            device,
            module,
        })
    }

    pub unsafe fn get_handle(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe { self.device.vk().destroy_shader_module(self.module, None) };
    }
}