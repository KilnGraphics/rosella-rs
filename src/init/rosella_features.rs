use std::any::Any;
use ash::Instance;
use paste::paste;
use crate::init::application_feature::{ApplicationDeviceFeature, ApplicationDeviceFeatureInstance, ApplicationInstanceFeature, InitResult};
use crate::init::instance::{InstanceConfigurator, InstanceInfo};
use crate::init::application_feature::FeatureBase;
use crate::init::device::{DeviceConfigurator, DeviceInfo};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::utils::FeatureAccess;
use crate::NamedUUID;
use crate::rosella::VulkanVersion;

pub fn register_rosella_headless(registry: &mut InitializationRegistry) {
    RosellaInstanceBase::register_into(registry, true);
    GetPhysicalDeviceProperties2::register_into(registry, false);

    RosellaDeviceBase::register_into(registry, true);
}

macro_rules! const_instance_feature{
    ($struct_name:ty, $name:literal, [$($dependency:expr),*]) => {
        impl $struct_name {
            const NAME: NamedUUID = NamedUUID::new_const($name);
            const DEPENDENCIES: &'static [NamedUUID] = &[$($dependency,)*];

            fn register_into(registry: &mut InitializationRegistry, required: bool) {
                registry.register_instance_feature(
                    Self::NAME,
                    Self::DEPENDENCIES.to_vec().into_boxed_slice(),
                    Box::new(Self::default()),
                    required
                )
            }
        }

        impl FeatureBase for $struct_name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
}

macro_rules! const_device_feature{
    ($struct_name:ident, $name:literal, [$($dependency:expr),*]) => {
        paste! {
            #[derive(Default)]
            pub struct [<$struct_name Generator>];

            impl ApplicationDeviceFeature for [<$struct_name Generator>] {
                fn make_instance(&self) -> Box<dyn ApplicationDeviceFeatureInstance> {
                    Box::new($struct_name::default())
                }
            }
        }

        impl $struct_name {
            const NAME: NamedUUID = NamedUUID::new_const($name);
            const DEPENDENCIES: &'static [NamedUUID] = &[$($dependency,)*];

            fn register_into(registry: &mut InitializationRegistry, required: bool) {
                registry.register_device_feature(
                    Self::NAME,
                    Self::DEPENDENCIES.to_vec().into_boxed_slice(),
                    paste! { Box::new([<$struct_name Generator>]::default()) },
                    required
                )
            }
        }

        impl FeatureBase for $struct_name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
}

#[derive(Default)]
pub struct RosellaInstanceBase;
const_instance_feature!(RosellaInstanceBase, "rosella:instance_base", []);

impl ApplicationInstanceFeature for RosellaInstanceBase {
    fn init(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo) -> InitResult {
        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, _: &mut InstanceConfigurator) {
    }

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

#[derive(Default)]
pub struct GetPhysicalDeviceProperties2;
const_instance_feature!(GetPhysicalDeviceProperties2, "rosella_vk:get_physical_device_properties_2", []);

impl ApplicationInstanceFeature for GetPhysicalDeviceProperties2 {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        if info.get_vulkan_version().is_supported(VulkanVersion::VK_1_1) {
            InitResult::Ok
        } else {
            if info.is_extension_supported::<ash::extensions::khr::GetPhysicalDeviceProperties2>() {
                InitResult::Ok
            } else {
                InitResult::Disable
            }
        }
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo, config: &mut InstanceConfigurator) {
        if !info.get_vulkan_version().is_supported(VulkanVersion::VK_1_1) {
            config.enable_extension::<ash::extensions::khr::GetPhysicalDeviceProperties2>();
        }
    }

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

pub struct WindowSurface {
    name: NamedUUID,
    extensions: Vec<std::ffi::CString>,
}

impl WindowSurface {
    pub fn new(window: &winit::window::Window) -> Self {
        let extensions = ash_window::enumerate_required_extensions(window).unwrap();

        Self {
            name: NamedUUID::new_const("rosella:window_surface"),
            extensions: extensions.into_iter().map(|str| std::ffi::CString::from(str)).collect()
        }
    }

    pub fn register_into(registry: &mut InitializationRegistry, window: &winit::window::Window, required: bool) -> NamedUUID {
        let instance = Box::new(Self::new(window));
        let name = instance.name.clone();

        registry.register_instance_feature(name.clone(), [].to_vec().into_boxed_slice(), instance, required);

        name
    }
}

impl FeatureBase for WindowSurface {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ApplicationInstanceFeature for WindowSurface {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &InstanceInfo) -> InitResult {
        for extension in &self.extensions {
            if !info.is_extension_supported_str(extension.to_str().unwrap()) {
                return InitResult::Disable
            }
        }
        InitResult::Ok
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, _: &InstanceInfo, config: &mut InstanceConfigurator) {
        for extension in &self.extensions {
            config.enable_extension_str_no_load(extension.to_str().unwrap())
        }
    }

    fn finish(self, _: &Instance) -> Option<Box<dyn Any>> {
        None
    }
}

#[derive(Default)]
struct RosellaDeviceBase;
const_device_feature!(RosellaDeviceBase, "rosella:device_base", []);

impl ApplicationDeviceFeatureInstance for RosellaDeviceBase {
    fn init(&mut self, _: &mut dyn FeatureAccess, info: &DeviceInfo) -> InitResult {
        todo!()
    }

    fn enable(&mut self, _: &mut dyn FeatureAccess, info: &DeviceInfo, config: &DeviceConfigurator) {
        todo!()
    }
}