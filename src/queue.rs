use ash::version::InstanceV1_0;
use ash::vk;
use std::default::Default;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
}
impl QueueFamilyIndices{
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}
impl Default for QueueFamilyIndices {
    fn default() -> Self {
        QueueFamilyIndices {
            graphics_family: None,
        }
    }
}



pub fn find_queue_families(
    instance: &ash::Instance,
    physical_device:& vk::PhysicalDevice,
) -> QueueFamilyIndices {
    let mut indices = QueueFamilyIndices {
        ..Default::default()
    };
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    for (i, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            indices.graphics_family = Some(i as u32);
        };
    }
    indices
}
