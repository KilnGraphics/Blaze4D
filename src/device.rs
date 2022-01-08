use std::sync::Arc;
use std::thread::JoinHandle;

use ash::vk;
use concurrent_queue::ConcurrentQueue;

use crate::init::EnabledFeatures;
use crate::instance::InstanceContext;
use crate::util::extensions::{AsRefOption, ExtensionFunctionSet, VkExtensionInfo, VkExtensionFunctions};
use crate::UUID;

struct WorkerData {
    queue: ConcurrentQueue<()>,
}

impl WorkerData {
    fn new() -> Self {
        Self {
            queue: ConcurrentQueue::unbounded(),
        }
    }

    /// After this function is called no new requests may be pushed into the worker
    pub fn close(&self) {
        self.queue.close();
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
    data: Arc<WorkerData>,
}

impl Worker {
    pub fn spawn() -> Self {
        let data = Arc::new(WorkerData::new());
        let cloned_data = data.clone();
        Self {
            thread: Some(std::thread::spawn(move|| Self::run(cloned_data))),
            data,
        }
    }

    fn run(data: Arc<WorkerData>) {

    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.data.close();
        self.thread.take().unwrap().join().unwrap();
    }
}

pub struct DeviceContextImpl {
    instance: InstanceContext,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    extensions: ExtensionFunctionSet,
    features: EnabledFeatures,
    worker: Worker,
}

impl Drop for DeviceContextImpl {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

#[derive(Clone)]
pub struct DeviceContext(Arc<DeviceContextImpl>);

impl DeviceContext {
    pub fn new(instance: InstanceContext, device: ash::Device, physical_device: vk::PhysicalDevice, extensions: ExtensionFunctionSet, features: EnabledFeatures) -> Self {
        Self(Arc::new(DeviceContextImpl{
            instance,
            device,
            physical_device,
            extensions,
            features,
            worker: Worker::spawn(),
        }))
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.0.instance.get_entry()
    }

    pub fn get_instance(&self) -> &InstanceContext {
        &self.0.instance
    }

    pub fn vk(&self) -> &ash::Device {
        &self.0.device
    }

    pub fn get_physical_device(&self) -> &vk::PhysicalDevice {
        &self.0.physical_device
    }

    pub fn get_extension<T: VkExtensionInfo>(&self) -> Option<&T> where VkExtensionFunctions: AsRefOption<T> {
        self.0.extensions.get()
    }

    pub fn is_extension_enabled(&self, uuid: UUID) -> bool {
        self.0.extensions.contains(uuid)
    }

    pub fn get_enabled_features(&self) -> &EnabledFeatures {
        &self.0.features
    }
}
