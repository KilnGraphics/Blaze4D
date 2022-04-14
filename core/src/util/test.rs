use crate::init::device::create_device;
use crate::init::InitializationRegistry;
use crate::init::instance::create_instance;
use crate::init::rosella_features::{register_rosella_debug, register_rosella_headless};
use crate::rosella::{DeviceContext, InstanceContextImpl};

pub fn make_headless_instance() -> InstanceContextImpl {
    let mut registry = InitializationRegistry::new();

    register_rosella_headless(&mut registry);
    register_rosella_debug(&mut registry, false);

    create_instance(&mut registry, "RosellaUnitTests", 1).unwrap()
}

pub fn make_headless_instance_device() -> (InstanceContextImpl, DeviceContext) {
    let mut registry = InitializationRegistry::new();

    register_rosella_headless(&mut registry);
    register_rosella_debug(&mut registry, false);

    let instance = create_instance(&mut registry, "RosellaUnitTests", 1).unwrap();
    let device = create_device(&mut registry, instance.clone(), &[]).unwrap();

    (instance, device)
}