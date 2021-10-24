use crate::init::initialization_registry::InitializationRegistry;

pub struct InstanceBuilder {
    registry: InitializationRegistry,
    enable_debug_utils: bool,
}

impl InstanceBuilder {

    pub fn new(registry: InitializationRegistry) -> Self {
        InstanceBuilder {
            registry,
            enable_debug_utils: true
        }
    }
}

