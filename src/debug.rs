use ash::extensions::ext;
use ash::version::EntryV1_0;
use ash::vk;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

// make a function to convert a raw string array to a string
fn raw_str_to_str(raw_str: &[c_char]) -> String {
    let p = raw_str.as_ptr();
    unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() }
}

pub fn check_validation_layer_support(entry: &ash::Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");
    if layer_properties.len() <= 0 {
        eprintln!("No available layers.");
        return false;
    } else {
        println!("Instance Available Layers: ");
        for layer in layer_properties.iter() {
            let layer_name = raw_str_to_str(&layer.layer_name);
            println!("\t{}", layer_name);
        }
    }

    println!("Checking required Validation Layers: ");
    let mut is_required_validation_layers_found = true;
    'outer: for required_validation_layer in VALIDATION.required_validation_layers.iter() {
        'inner: for layer_property in layer_properties.iter() {
            let layer_name = raw_str_to_str(&layer_property.layer_name);
            if layer_name == *(required_validation_layer) {
                println!("\t{} is available", required_validation_layer);
                continue 'outer;
            }
        }
        is_required_validation_layers_found = false;
        eprintln!("\t{} is not available", required_validation_layer);
    }
    is_required_validation_layers_found
}

pub unsafe extern "system" fn debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}
pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT{
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::all(),
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
            pfn_user_callback: Some(debug_utils_callback),
            p_user_data: ptr::null_mut(),
            ..Default::default()
        }
    }
pub fn setup_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Option<(vk::DebugUtilsMessengerEXT)> {
    if VALIDATION.is_enable {
        let create_info = populate_debug_messenger_create_info();
        let debug_utils_messenger = unsafe {
            ext::DebugUtils::new(entry, instance)
                .create_debug_utils_messenger(&create_info, None)
                .expect("Debug Utils Callback not found")
        };
        Some((debug_utils_messenger))
    } else {
        None
    }
}

pub fn destroy_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
) {
    match debug_utils_messenger {
        Some(debug_utils_messenger) => unsafe {
            ext::DebugUtils::new(entry, instance)
                .destroy_debug_utils_messenger(debug_utils_messenger, None);
        },
        None => {}
    }
}
