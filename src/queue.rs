use ash::version::InstanceV1_0;
use ash::vk;
use std::default::Default;
use std::collections::HashSet;
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}
impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
    pub fn iter(&self) -> impl Iterator<Item = u32> {
        vec![self.graphics_family.unwrap(), self.present_family.unwrap()].into_iter()
    }
    pub fn into_unique(&self) -> HashSet<u32> {
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(self.graphics_family.unwrap());
        unique_queue_families.insert(self.present_family.unwrap());
        unique_queue_families
    }
}
impl Default for QueueFamilyIndices {
    fn default() -> Self {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }
}

pub fn find_queue_families(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> QueueFamilyIndices {
    let entry = ash::Entry::new().unwrap();
    let surface_loader = ash::extensions::khr::Surface::new(&entry,instance);
    let mut indices = QueueFamilyIndices {
        ..Default::default()
    };
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        for (i,queue_family) in queue_families.iter().enumerate(){
            if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }
            let is_present_support = unsafe {surface_loader.get_physical_device_surface_support(*physical_device,i as u32,*surface)};
            if is_present_support {
                indices.present_family = Some(i as u32);
            }
            if indices.is_complete() {
                break;
            }
        }
    indices
}
