use crate::queue::find_queue_families;
use crate::surface;
use ash::version::InstanceV1_0;
use ash::vk;

use crate::debug::VALIDATION;
use crate::physical_device::physical_device_extensions;
use std::default::Default;
use std::ffi::CString;
use std::ptr;
use std::collections::HashSet;
pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> ash::Device {
    let indices = find_queue_families(instance, physical_device, surface);
    let unique_queue_families= indices.into_unique();
    let mut queue_create_infos :Vec<vk::DeviceQueueCreateInfo> = vec![];
    let queue_priority = 1.0f32;
    for (_i, queue_family) in unique_queue_families.iter().enumerate() {
        let queue_create_info = vk::DeviceQueueCreateInfo{
            queue_family_index: queue_family.clone(),
            queue_count: 1,
            p_queue_priorities: &queue_priority,
            ..Default::default()
        };
        queue_create_infos.push(queue_create_info);
    }


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
    let device_extensions_raw_names :Vec<CString> = physical_device_extensions.iter().map(|extension| {
        CString::new(*extension).unwrap()
    }).collect();
    let device_extensions_names :Vec<*const i8> = device_extensions_raw_names.iter().map(|extension| {
        extension.as_ptr()
    }).collect();
    let device_create_info = vk::DeviceCreateInfo {
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
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
        enabled_extension_count :physical_device_extensions.len() as u32,
        pp_enabled_extension_names: device_extensions_names.as_ptr(),
        ..Default::default()
    };
    unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create logical device")
    }
}
// pub struct SurfaceStuff {
//     pub surface_loader: ash::extensions::khr::Surface,
//     pub surface: vk::SurfaceKHR,

//     pub screen_width: u32,
//     pub screen_height: u32,
// }

// pub fn create_logical_device(
//     instance: &ash::Instance,
//     physical_device: vk::PhysicalDevice,
//     validation: &super::debug::ValidationInfo,
//     device_extensions: &DeviceExtension,
//     surface_stuff: &SurfaceStuff,
// ) -> (ash::Device, QueueFamilyIndices) {
//     let indices = find_queue_family(instance, physical_device, surface_stuff);

//     use std::collections::HashSet;
//     let mut unique_queue_families = HashSet::new();
//     unique_queue_families.insert(indices.graphics_family.unwrap());
//     unique_queue_families.insert(indices.present_family.unwrap());

//     let queue_priorities = [1.0_f32];
//     let mut queue_create_infos = vec![];
//     for &queue_family in unique_queue_families.iter() {
//         let queue_create_info = vk::DeviceQueueCreateInfo {
//             s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
//             p_next: ptr::null(),
//             flags: vk::DeviceQueueCreateFlags::empty(),
//             queue_family_index: queue_family,
//             p_queue_priorities: queue_priorities.as_ptr(),
//             queue_count: queue_priorities.len() as u32,
//         };
//         queue_create_infos.push(queue_create_info);
//     }

//     let physical_device_features = vk::PhysicalDeviceFeatures {
//         sampler_anisotropy: vk::TRUE, // enable anisotropy device feature from Chapter-24.
//         ..Default::default()
//     };

//     let requred_validation_layer_raw_names: Vec<CString> = validation
//         .required_validation_layers
//         .iter()
//         .map(|layer_name| CString::new(*layer_name).unwrap())
//         .collect();
//     let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
//         .iter()
//         .map(|layer_name| layer_name.as_ptr())
//         .collect();

//     let enable_extension_names = device_extensions.get_extensions_raw_names();

//     let device_create_info = vk::DeviceCreateInfo {
//         s_type: vk::StructureType::DEVICE_CREATE_INFO,
//         p_next: ptr::null(),
//         flags: vk::DeviceCreateFlags::empty(),
//         queue_create_info_count: queue_create_infos.len() as u32,
//         p_queue_create_infos: queue_create_infos.as_ptr(),
//         enabled_layer_count: if validation.is_enable {
//             enable_layer_names.len()
//         } else {
//             0
//         } as u32,
//         pp_enabled_layer_names: if validation.is_enable {
//             enable_layer_names.as_ptr()
//         } else {
//             ptr::null()
//         },
//         enabled_extension_count: enable_extension_names.len() as u32,
//         pp_enabled_extension_names: enable_extension_names.as_ptr(),
//         p_enabled_features: &physical_device_features,
//     };

//     let device: ash::Device = unsafe {
//         instance
//             .create_device(physical_device, &device_create_info, None)
//             .expect("Failed to create logical Device!")
//     };

//     (device, indices)
// }