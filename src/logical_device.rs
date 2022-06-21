use crate::queue::find_queue_families;
use ash::version::InstanceV1_0;
use ash::vk;

use crate::debug::VALIDATION;
use std::default::Default;
use std::ffi::CString;
use std::ptr;
pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
) -> ash::Device {
    let indices = find_queue_families(instance, physical_device);
    let queue_priority = 1.0f32;
    let queue_create_info = vk::DeviceQueueCreateInfo {
        queue_family_index: indices.graphics_family.unwrap(),
        queue_count: 1,
        p_queue_priorities: &queue_priority,
        ..Default::default()
    };

    let physical_device_features = vk::PhysicalDeviceFeatures {
        ..Default::default()
    };
    let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let device_create_info = vk::DeviceCreateInfo {
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_create_info,
        p_enabled_features: &physical_device_features,
        pp_enabled_layer_names: if VALIDATION.is_enable {
            enable_layer_names.as_ptr()
        } else {
            ptr::null()
        },
        enabled_layer_count: if VALIDATION.is_enable {
            enable_layer_names.len()
        } else {
            0
        } as u32,
        ..Default::default()
    };
    unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create logical device")
    }
}
