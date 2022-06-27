use crate::queue::{find_queue_families, QueueFamilyIndices};
use crate::surface::SurfaceStuff;
use crate::swapchain::{query_swapchain_support, SwapChainSupportDetails};
use ash::version::InstanceV1_0;
use ash::vk;
use std::default::Default;
use std::ffi::CStr;

pub const physical_device_extensions: [&'static str; 1] = ["VK_KHR_swapchain"];

pub fn pick_pyhsical_device(
    instance: &ash::Instance,
    surface: &SurfaceStuff,
) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices.")
    };
    // for physical_device in physical_devices.iter() {
    //     let physical_device_properties =
    //     unsafe { instance.get_physical_device_properties(*physical_device) };
    //     unsafe{   println!("{:?}", std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr()));}   
    //     if(  unsafe{  std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr()).to_str().unwrap()} == "Intel(R) UHD Graphics 630")
    //     {
    //         return *physical_device;
    //     }
    // }
    for physical_device in physical_devices.iter() {

        if is_device_suitable(instance, *physical_device, surface) {
            return *physical_device;
        }
      
    }
    panic!("Failed to find a suitable GPU!");
}
fn is_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: &SurfaceStuff,
) -> bool {
    let physical_device_properties =
        unsafe { instance.get_physical_device_properties(physical_device) };
    let physical_device_features =
        unsafe { instance.get_physical_device_features(physical_device) };
        if check_device_suitable(instance, physical_device) &&find_queue_families(instance, &physical_device, &surface.surface).is_complete(){
                let swap_chain_support = query_swapchain_support(physical_device, surface);
                return swap_chain_support.formats.len() > 0 && swap_chain_support.present_modes.len() > 0
            }
        false
    

}

fn check_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let physical_device_available_extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device) };
    match physical_device_available_extensions {
        Ok(available_extensions) => {
            for required_extension in physical_device_extensions.iter(){
                if ! (available_extensions.iter().any(|extension| {
                   unsafe{ CStr::from_ptr(extension.extension_name.as_ptr())}
                        .to_str()
                        .unwrap()
                        == *required_extension})){
                    return false;
                        }
                
            } 
            true
        }
        Err(err) => {
            panic!("Failed to enumerate device extensions: {:?}", err);
        }
    }
}
