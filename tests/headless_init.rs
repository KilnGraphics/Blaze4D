use rosella_rs::init::device::create_device;
use rosella_rs::init::InitializationRegistry;
use rosella_rs::init::instance::create_instance;
use rosella_rs::init::rosella_features::register_rosella_headless;

mod test_common;

#[test]
fn init_no_feature() {
    let mut registry = InitializationRegistry::new();
    let instance_context = match create_instance(&mut registry, "Rosella Test", 1) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to create instance {:?}", err);
        }
    };

    #[allow(unused)]
    let device_context = match create_device(&mut registry, instance_context.clone(), &[]) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to create device {:?}", err);
        }
    };
}

#[test]
fn init_rosella() {
    let mut registry = InitializationRegistry::new();
    register_rosella_headless(&mut registry);

    let instance_context = match create_instance(&mut registry, "Rosella Test", 1) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to create instance {:?}", err);
        }
    };

    #[allow(unused)]
    let device_context = match create_device(&mut registry, instance_context.clone(), &[]) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to create device {:?}", err);
        }
    };
}