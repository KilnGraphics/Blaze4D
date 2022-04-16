use std::ffi::CStr;
use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT};

pub trait DebugMessengerCallback {
    fn on_message(
        &self,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        message: &CStr,
        data: &vk::DebugUtilsMessengerCallbackDataEXT,
    );
}

pub struct RustLogDebugMessenger {
}

impl RustLogDebugMessenger {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl DebugMessengerCallback for RustLogDebugMessenger {
    fn on_message(&self, message_severity: DebugUtilsMessageSeverityFlagsEXT, _: DebugUtilsMessageTypeFlagsEXT, message: &CStr, _: &DebugUtilsMessengerCallbackDataEXT) {
        if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            log::error!("{:?}", message);
        } else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::WARNING) {
            log::warn!("{:?}", message);
        } else if message_severity.contains(DebugUtilsMessageSeverityFlagsEXT::INFO) {
            log::info!("{:?}", message);
        } else {
            log::info!("Unknown severity: {:?}", message);
        }
    }
}