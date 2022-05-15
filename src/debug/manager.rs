use std::ops::Deref;
use std::sync::{Arc, Mutex};
use ash::prelude::VkResult;
use ash::vk;
use winit::event::VirtualKeyCode::Tab;
use crate::debug::target::TargetShare;

use crate::prelude::*;
use crate::device::device::VkQueue;

use super::target::Target;
use super::target::TargetDrawGlobals;

#[derive(Clone)]
pub struct DebugOverlay(Arc<DebugOverlayImpl>);

impl DebugOverlay {
    pub fn new(device: DeviceEnvironment) -> Self {
        Self(Arc::new(DebugOverlayImpl::new(device)))
    }

    pub fn create_target(&self, initial_size: Vec2u32) -> Target {
        Target::new(Arc::new(Mutex::new(TargetShare::new(initial_size))), self.0.clone())
    }
}

pub(super) struct DebugOverlayImpl {
    device: DeviceEnvironment,
    target_draw_globals: TargetDrawGlobals,
}

impl DebugOverlayImpl {
    fn new(device: DeviceEnvironment) -> Self {
        let target_draw_globals = TargetDrawGlobals::new(&device, device.get_device().get_main_queue()).unwrap();

        Self {
            device,
            target_draw_globals,
        }
    }

    pub fn get_device(&self) -> &DeviceEnvironment {
        &self.device
    }

    pub fn get_target_draw_globals(&self) -> &TargetDrawGlobals {
        &self.target_draw_globals
    }
}

impl Drop for DebugOverlayImpl {
    fn drop(&mut self) {
        self.target_draw_globals.destroy(&self.device);
    }
}

#[derive(Debug)]
pub(super) enum ResourceResetError {
    OutOfHostMemory,
    OutOfDeviceMemory,
    DeviceLost,
}

impl ResourceResetError {
    pub fn try_from(result: vk::Result) -> Option<Self> {
        match result {
            vk::Result::ERROR_OUT_OF_HOST_MEMORY => Some(Self::OutOfHostMemory),
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Some(Self::OutOfDeviceMemory),
            vk::Result::ERROR_DEVICE_LOST => Some(Self::DeviceLost),
            _ => None
        }
    }
}

impl From<vk::Result> for ResourceResetError {
    fn from(result: vk::Result) -> Self {
        match result {
            vk::Result::ERROR_OUT_OF_HOST_MEMORY => Self::OutOfHostMemory,
            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Self::OutOfDeviceMemory,
            vk::Result::ERROR_DEVICE_LOST => Self::DeviceLost,
            _ => panic!("Unexpected result: {:?}", result)
        }
    }
}