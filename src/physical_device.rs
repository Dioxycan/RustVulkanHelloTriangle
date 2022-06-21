use crate::queue::{QueueFamilyIndices,find_queue_families};
use std::default::Default;

use ash::version::InstanceV1_0;
use ash::vk;

pub fn pick_pyhsical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices.")
    };

    for physical_device in physical_devices.iter() {
        if is_device_suitable(instance, *physical_device) {
            return *physical_device;
        }
    }
    panic!("Failed to find a suitable GPU!");
}
fn is_device_suitable(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool {
    let physical_device_properties =
        unsafe { instance.get_physical_device_properties(physical_device) };
    let physical_device_features =
        unsafe { instance.get_physical_device_features(physical_device) };

    find_queue_families(instance, &physical_device).is_complete()
}
