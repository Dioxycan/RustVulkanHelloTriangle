use ash::extensions::{ext, khr};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk_make_version;
use ash::Entry;

use std::ffi::{CString, c_void};
use std::ptr;
use crate::debug::{
    check_validation_layer_support, populate_debug_messenger_create_info, VALIDATION,
};

pub fn create_instance(entry: &Entry) -> ash::Instance {
    if VALIDATION.is_enable && check_validation_layer_support(entry) == false {
        panic!("Validation layers requested, but not available!");
    }

    let app_name = CString::new("Hello").unwrap();
    let engine_name = CString::new("Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        application_version: vk_make_version!(1, 0, 0),
        p_application_name: app_name.as_ptr(),
        engine_version: vk_make_version!(1, 0, 0),
        p_engine_name: engine_name.as_ptr(),
        api_version: vk_make_version!(1, 0, 0),
        ..Default::default()
    };

    let required_extensions: Vec<*const i8> = vec![
        khr::Surface::name().as_ptr(),
        khr::Win32Surface::name().as_ptr(),
        ext::DebugUtils::name().as_ptr(),
    ];

    let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,

        enabled_extension_count: required_extensions.len() as u32,
        pp_enabled_extension_names: required_extensions.as_ptr(),

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
        p_next: if VALIDATION.is_enable {
            &populate_debug_messenger_create_info() as *const vk::DebugUtilsMessengerCreateInfoEXT
                as *const c_void
        } else {
            ptr::null()
        },
        ..Default::default()
    };
    let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };
    instance
}
