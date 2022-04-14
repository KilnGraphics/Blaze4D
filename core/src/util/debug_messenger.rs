use std::ffi::CStr;
use ash::vk;

pub trait DebugMessengerCallback {
    fn on_message(
        &self,
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        message: &CStr,
        data: &vk::DebugUtilsMessengerCallbackDataEXT,
    );
}

pub struct StdOutDebugMessenger {
}