use std::collections::HashMap;
use ash::{Entry, Instance};
use crate::NamedUUID;
use paste::paste;
use crate::util::id::UUID;

#[derive(Clone)]
pub struct ExtensionFunctionSet {
    functions: HashMap<UUID, VkExtensionFunctions>,
}

impl ExtensionFunctionSet {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn add<T: VkExtensionInfo>(&mut self, functions: Box<T>) where VkExtensionFunctions: From<Box<T>> {
        if self.functions.insert(T::UUID.get_uuid(), VkExtensionFunctions::from(functions)).is_some() {
            panic!("Added already existing function set");
        }
    }

    pub fn contains(&self, uuid: UUID) -> bool {
        self.functions.contains_key(&uuid)
    }

    pub fn get<T: VkExtensionInfo>(&self) -> Option<&T> where VkExtensionFunctions: AsRefOption<T> {
        self.functions.get(&T::UUID.get_uuid()).map(|v| v.as_ref_option().expect("Extension type mismatch"))
    }
}

pub trait VkExtensionInfo {
    const UUID: NamedUUID;
}

pub type InstanceExtensionLoaderFn = dyn Fn(&mut ExtensionFunctionSet, &ash::Entry, &ash::Instance);

pub trait InstanceExtensionLoader {
    fn load_extension(function_set: &mut ExtensionFunctionSet, entry: &ash::Entry, instance: &ash::Instance);
}

pub trait AsRefOption<T> {
    fn as_ref_option(&self) -> Option<&T>;
}

macro_rules! make_vk_extension_info {
    ($($struct_name:ty, $string_name:ident);+) => {
        paste! {
            #[derive(Clone)]
            pub enum VkExtensionFunctions {
                $([<$string_name:lower:camel>](Box<$struct_name>),)+
            }

            impl VkExtensionFunctions {
                $(
                pub fn [<from_ $string_name:lower>](obj: Box<$struct_name>) -> Self {
                    Self::[<$string_name:lower:camel>](obj)
                }

                pub fn [<get_ $string_name:lower>](&self) -> Option<&$struct_name> {
                    match &self {
                        Self::[<$string_name:lower:camel>](obj) => Some(obj.as_ref()),
                        _ => None,
                    }
                }
                )+
            }

            $(
            impl From<Box<$struct_name>> for VkExtensionFunctions {
                fn from(obj: Box<$struct_name>) -> Self {
                    Self::[<$string_name:lower:camel>](obj)
                }
            }

            impl AsRefOption<$struct_name> for VkExtensionFunctions {
                fn as_ref_option(&self) -> Option<&$struct_name> {
                    match &self {
                        Self::[<$string_name:lower:camel>](obj) => Some(obj.as_ref()),
                        _ => None,
                    }
                }
            }
            )+
        }

        $(impl VkExtensionInfo for $struct_name {
            const UUID: NamedUUID = NamedUUID::new_const(stringify!($string_name));
        })+
    }
}

make_vk_extension_info!(
    ash::extensions::khr::Swapchain, VK_KHR_Swapchain;
    ash::extensions::khr::GetPhysicalDeviceProperties2, VK_KHR_get_physical_device_properties2
);

impl InstanceExtensionLoader for ash::extensions::khr::GetPhysicalDeviceProperties2 {
    fn load_extension(function_set: &mut ExtensionFunctionSet, entry: &Entry, instance: &Instance) {
        function_set.add(Box::new(ash::extensions::khr::GetPhysicalDeviceProperties2::new(entry, instance)))
    }
}