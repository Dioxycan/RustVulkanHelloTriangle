use crate::queue::find_queue_families;
use crate::{physical_device, surface};
use ash::extensions::khr;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use num::clamp;
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swap_chain_support(
    instance: &ash::Instance,
    surface: &vk::SurfaceKHR,
    physical_device: &vk::PhysicalDevice,
) -> SwapChainSupportDetails {
    let surface_loader = khr::Surface::new(&ash::Entry::new().unwrap(), instance);
    let details = SwapChainSupportDetails {
        capabilities: unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*physical_device, *surface)
                .expect("Failed to get surface capabilities")
        },
        formats: unsafe {
            surface_loader
                .get_physical_device_surface_formats(*physical_device, *surface)
                .expect("Failed to get surface formats")
        },
        present_modes: unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*physical_device, *surface)
                .expect("Failed to get surface present modes")
        },
    };
    details
}
fn choose_swap_surface_format(
    availeble_formats: Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR {
    for availeble_format in availeble_formats.iter() {
        if (availeble_format.format == vk::Format::B8G8R8A8_SRGB
            && availeble_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
        {
            return *availeble_format;
        }
    }
    availeble_formats[0]
}
fn choose_swap_present_mode(
    availeble_present_modes: Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
    for availeble_present_mode in availeble_present_modes.iter() {
        if *availeble_present_mode == vk::PresentModeKHR::MAILBOX {
            return *availeble_present_mode;
        }
    }
    vk::PresentModeKHR::FIFO
}
fn choose_swap_extent(
    capablitie: vk::SurfaceCapabilitiesKHR,
    window: &winit::window::Window,
) -> vk::Extent2D {
    if capablitie.current_extent.width != std::u32::MAX {
        return capablitie.current_extent;
    } else {
        let (width, height): (u32, u32) = window.inner_size().into();

        vk::Extent2D {
            width: clamp(
                width,
                capablitie.min_image_extent.width,
                capablitie.max_image_extent.width,
            ),
            height: clamp(
                height,
                capablitie.min_image_extent.height,
                capablitie.max_image_extent.height,
            ),
        }
    }
}
fn create_swap_chain_image_views(device:&ash::Device,swap_chain_images:&Vec<vk::Image>,swap_chain_image_format:&vk::Format)->Vec<vk::ImageView>{
    let mut image_views = vec![];
    for image in swap_chain_images.iter(){
        let image_view_create_info = vk::ImageViewCreateInfo{
            image: *image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: *swap_chain_image_format,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange{
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        let image_view = unsafe {
            device.create_image_view(&image_view_create_info, None).expect("Failed to create image view")
        };
        image_views.push(image_view);
        
    }
    image_views

}
pub fn create_swap_chain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
    window: &winit::window::Window,
) ->(vk::SwapchainKHR,khr::Swapchain,Vec<vk::Image>,vk::Format, vk::Extent2D,Vec<vk::ImageView>) {
    let swap_chain_support = query_swap_chain_support(&instance, &surface, &physical_device);
    let surface_format = choose_swap_surface_format(swap_chain_support.formats);
    let present_mode = choose_swap_present_mode(swap_chain_support.present_modes);
    let extent = choose_swap_extent(swap_chain_support.capabilities, &window);

    let image_count = if swap_chain_support.capabilities.max_image_count > 0
        && swap_chain_support.capabilities.min_image_count + 1
            > swap_chain_support.capabilities.max_image_count
    {
        swap_chain_support.capabilities.max_image_count
    } else {
        swap_chain_support.capabilities.min_image_count + 1
    };
    let indices = find_queue_families(instance, physical_device, surface);
    let queue_family_indices = [indices.graphics_family.unwrap(), indices.present_family.unwrap()];

    let swap_chain_create_info = vk::SwapchainCreateInfoKHR{
        surface: *surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode: if indices.graphics_family != indices.present_family {
            vk::SharingMode::CONCURRENT
        }else{
            vk::SharingMode::EXCLUSIVE
        },
        queue_family_index_count: if indices.graphics_family != indices.present_family {
            2
        }else{
            1
        },
        p_queue_family_indices: if indices.graphics_family != indices.present_family {
            queue_family_indices.as_ptr()
        }else{
            std::ptr::null()
        },
        pre_transform: swap_chain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode: present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        ..Default::default()
    };
    let swap_chain_loader = khr::Swapchain::new(instance,device);
    let swap_chain = unsafe {
        swap_chain_loader
            .create_swapchain(
                &swap_chain_create_info,
                None,
            )
            .expect("Failed to create swap chain")
    };
    let swap_chain_images = unsafe {
        swap_chain_loader
            .get_swapchain_images(swap_chain)
            .expect("Failed to get swap chain images")
    };
    let swap_chain_image_views = create_swap_chain_image_views(device, &swap_chain_images, &surface_format.format);
    (swap_chain,swap_chain_loader,swap_chain_images,surface_format.format,extent,swap_chain_image_views)

}
