use ash::extensions::ext;
use ash::extensions::khr;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::PresentModeKHR;
use ash::vk::{self, NvShaderSubgroupPartitionedFn};
use ash::vk_make_version;
use std::collections::hash_set::HashSet;
use std::default::Default;
use std::f32::consts::PI;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::Arc;
use std::u32;
use winit::{self, event_loop};
pub const physical_device_extensions: [&'static str; 1] = ["VK_KHR_swapchain"];
pub const vert_file_path_const: &'static str = "src/shaders/shader.vert.spv";
pub const frag_file_path_const: &'static str = "src/shaders/shader.frag.spv";
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;


struct SwapChainSupportDetails {
    capabilities: Option<vk::SurfaceCapabilitiesKHR>,
    formats: Option<Vec<vk::SurfaceFormatKHR>>,
    present_modes: Option<Vec<vk::PresentModeKHR>>,
}
impl SwapChainSupportDetails {
    fn query_swapchain_support(
        physical_device: &vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: &vk::SurfaceKHR
    ) -> Self {
        SwapChainSupportDetails {
            capabilities: match unsafe {
                surface_loader
                    .get_physical_device_surface_capabilities(
                        *physical_device,
                        *surface,
                    )
            } {
                Ok(capabilities) => Some(capabilities),
                Err(err) => {
                    eprintln!("Failed to get surface capabilities{:?}", err);
                    None
                }
            },
            formats: match unsafe {
                surface_loader
                    .get_physical_device_surface_formats(*physical_device, *surface)
            } {
                Ok(formats) => Some(formats),
                Err(err) => {
                    eprintln!("Failed to get surface formats{:?}", err);
                    None
                }
            },
            present_modes: match unsafe {
                    surface_loader
                    .get_physical_device_surface_present_modes(
                        *physical_device,
                        *surface,
                    )
            } {
                Ok(present_modes) => Some(present_modes),
                Err(err) => {
                    eprintln!("Failed to get surface present modes{:?}", err);
                    None
                }
            },
        }
    }
    fn is_Suitable(&self) -> bool {
        self.formats.is_some() && self.present_modes.is_some()
    }
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}
impl QueueFamilyIndices {
    fn new() -> Self {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
    pub fn iter(&self) -> impl Iterator<Item = u32> {
        vec![self.graphics_family.unwrap(), self.present_family.unwrap()].into_iter()
    }
    fn to_unique(&self) -> HashSet<u32> {
        let mut set = HashSet::new();
        for i in self.iter() {
            set.insert(i);
        }
        set
    }
}
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
        for layer_property in layer_properties.iter() {
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
    vk::DebugUtilsMessengerCreateInfoEXT {
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_utils_callback),
        p_user_data: ptr::null_mut(),
        ..Default::default()
    }
}
pub fn setup_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Option<vk::DebugUtilsMessengerEXT> {
    if VALIDATION.is_enable {
        let create_info = populate_debug_messenger_create_info();
        let debug_utils_messenger = unsafe {
            ext::DebugUtils::new(entry, instance)
                .create_debug_utils_messenger(&create_info, None)
                .expect("Debug Utils Callback not found")
        };
        Some(debug_utils_messenger)
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





fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    let available_extensions =
        unsafe { instance.enumerate_device_extension_properties(*physical_device) };

    match available_extensions {
        Ok(available_extensions) => {
            for required_extension in physical_device_extensions.iter() {
                if !(available_extensions.iter().any(|extension| {
                    unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) }
                        .to_str()
                        .unwrap()
                        == *required_extension
                })) {
                    return false;
                }
            }
            true
        }
        Err(err) => {
            eprintln!("Failed to enumerate device extensions: {:?}", err);
            false
        }
    }
}

fn find_queue_families(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: &vk::SurfaceKHR,
) -> QueueFamilyIndices {
    let mut indices = QueueFamilyIndices::new();
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
    for (i, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            indices.graphics_family = Some(i as u32);
        }
        let is_present_support = unsafe {
            surface_loader.get_physical_device_surface_support(*physical_device, i as u32, *surface)
        };
        if is_present_support {
            indices.present_family = Some(i as u32);
        }
        if indices.is_complete() {
            break;
        }
    }
    indices
}

fn is_device_suitable(
    instance: &ash::Instance,
    physical_device: &vk::PhysicalDevice,
    surface_loader: &khr::Surface,
    surface: &vk::SurfaceKHR,
) -> bool {
    let indices = find_queue_families(instance, physical_device, surface_loader, surface);
    let extensions_supported = check_device_extension_support(instance, physical_device);
    let swapchain_adequate = SwapChainSupportDetails::query_swapchain_support(
        physical_device,
        surface_loader,
        surface,
    );
    let supported_features = unsafe {
        instance
            .get_physical_device_features(*physical_device)
    };

    indices.is_complete()
        && extensions_supported
        && swapchain_adequate.is_Suitable()
        && supported_features.sampler_anisotropy as u32 == 1
}


fn choose_swap_surface_format(swapchain_support:&SwapChainSupportDetails) -> vk::SurfaceFormatKHR {
    match &swapchain_support.formats {
        Some(formats) => {
            for format in formats.iter() {
                if format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                {
                    return *format;
                }
            }
            panic!("failed to choose swap surface format");
        }
        _ => {
            panic!("failed to choose swap surface format");
        }
    }
}
fn choose_swap_present_mode(swapchain_support:&SwapChainSupportDetails) -> vk::PresentModeKHR {
    match &swapchain_support.present_modes {
        Some(modes) => {
            for mode in modes.iter() {
                if *mode == vk::PresentModeKHR::MAILBOX {
                    println!("Present mode: Mailbox");
                    return *mode;
                }
            }
            // for mode in modes.iter(){
            //     if *mode == vk::PresentModeKHR::IMMEDIATE{
            //         println!("Present mode: Immediate");
            //         return *mode;
            //     }
            // }
            println!("Present mode: FIFO");
            return vk::PresentModeKHR::FIFO;
        }
        _ => {
            panic!("failed to choose swap present mode");
        }
    }
}
fn choose_swap_extent(swapchain_support:&SwapChainSupportDetails,window_extent:&mut vk::Extent2D) -> vk::Extent2D {
    match swapchain_support.capabilities {
        Some(capabilities) => {
            if capabilities.current_extent.width != std::u32::MAX {
                return capabilities.current_extent;
            } else {
                let mut actual_extent = window_extent;
                actual_extent.width = std::cmp::max(
                    capabilities.min_image_extent.width,
                    std::cmp::min(capabilities.max_image_extent.width, actual_extent.width),
                );
                actual_extent.height = std::cmp::max(
                    capabilities.min_image_extent.height,
                    std::cmp::min(capabilities.max_image_extent.height, actual_extent.height),
                );
                return *actual_extent;
            }
        }
        _ => {
            panic!("failed to choose swap extent");
        }
    }
}

fn main(){
    let width = 800;
    let height = 600;
    let event_loop = winit::event_loop::EventLoop::new();
    let entry = ash::Entry::new().expect("Failed to create entry");
    let mut window_extent = vk::Extent2D {
        width: width as u32,
        height: height as u32,
    };
    let window = winit::window::WindowBuilder::new()
        .with_title("hello")
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(&event_loop)
        .expect("Failed to create window");

    let required_extensions = vec![
        khr::Surface::name().as_ptr(),
        khr::Win32Surface::name().as_ptr(),
        ext::DebugUtils::name().as_ptr(),
    ];

    if VALIDATION.is_enable && check_validation_layer_support(&entry) == false {
        panic!("Validation layers requested, but not available!");
    }
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&unsafe { CStr::from_ptr("Vulkan".as_ptr() as *const i8) })
        .application_version(vk_make_version!(1, 0, 0))
        .engine_name(&unsafe { CStr::from_ptr("No Engine".as_ptr() as *const i8) })
        .engine_version(vk_make_version!(1, 0, 0))
        .api_version(vk_make_version!(1, 0, 0));
    let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let instance_create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info.build(),

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

    let instance = unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .expect("Failed to create instance")
    };
    let debug_utils_loader = ext::DebugUtils::new(&entry,&instance);

    let debug_messenger = if VALIDATION.is_enable {
        let create_info = populate_debug_messenger_create_info();
        let debug_utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&create_info, None)
                .expect("Debug Utils Callback not found")
        };
       Some(debug_utils_messenger)
    } else {
       None
    };

    let surface_loader = khr::Surface::new(
        &entry,
        &instance,
    );
    let surface =  unsafe {
        use std::os::raw::c_void;
        use std::ptr;
        use winapi::shared::windef::HWND;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd() as HWND;
        let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance,
            hwnd: hwnd as *const c_void,
        };
        let win32_surface_loader = khr::Win32Surface::new(
            &entry,
            &instance,
        );
        win32_surface_loader
            .create_win32_surface(&win32_create_info, None)
            .expect("Failed to create win32 surface")
    };
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };
    let mut physical_device :vk::PhysicalDevice = vk::PhysicalDevice::null();  
    for device in physical_devices.iter() {

        if is_device_suitable(&instance, &device, &surface_loader, &surface) {
            physical_device = *device;
            break;
        }
    }
    unsafe {
        let properties = instance
            .get_physical_device_properties(physical_device);
        println!(
            "Physical Device: {}",
            CStr::from_ptr(properties.device_name.as_ptr())
                .to_str()
                .unwrap()
        );
    }


    let indices = find_queue_families(&instance, &physical_device, &surface_loader,&surface);
    let unique_queue_families = indices.to_unique();
    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }
    let device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .build();
    let mut create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_features(&device_features)
        .build();
    if VALIDATION.is_enable {
        create_info.enabled_layer_count = VALIDATION.required_validation_layers.len() as u32;
        create_info.pp_enabled_layer_names = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap().as_ptr())
            .collect::<Vec<*const i8>>()
            .as_ptr();
    } else {
        create_info.enabled_layer_count = 0;
        create_info.pp_enabled_layer_names = ptr::null();
    }
    let device = unsafe {
        instance
            .create_device(physical_device, &create_info, None)
            .expect("Failed to create logical device")
    };

  let graphic_queue = unsafe {
    
            device
            .get_device_queue(indices.graphics_family.unwrap(), 0)
      
    };
    let present_queue = unsafe {
    
        device
            .get_device_queue(indices.present_family.unwrap(), 0)  
    };
    let queue_family_indices = find_queue_families(&instance, &physical_device, &surface_loader,&surface);
    let pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family.unwrap())
        .build();
    let command_pool = unsafe {
        device
            .create_command_pool(&pool_info, None)
            .expect("Failed to create command pool")
    };
    let swapchain_support = SwapChainSupportDetails::query_swapchain_support(
        &physical_device,
        &surface_loader,
        &surface,
    );
    let surface_format = choose_swap_surface_format(&swapchain_support);
    let present_mode = choose_swap_present_mode(&swapchain_support);
    let extent = choose_swap_extent(&swapchain_support, &mut window_extent);
    let mut image_count =
        swapchain_support
        .capabilities
        .unwrap()
        .min_image_count
        + 1;
    if 
        swapchain_support
        .capabilities
        .unwrap()
        .max_image_count
        > 0
        && image_count
            > swapchain_support
                .capabilities
                .unwrap()
                .max_image_count
    {
        image_count = swapchain_support
            .capabilities
            .unwrap()
            .max_image_count;
    }
    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        ..Default::default()
    };
    let indices = [
        queue_family_indices.graphics_family.unwrap(),
        queue_family_indices.present_family.unwrap(),
    ];
    if (queue_family_indices.graphics_family != queue_family_indices.present_family) {
        swapchain_create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
        swapchain_create_info.queue_family_index_count = 2;
        swapchain_create_info.p_queue_family_indices = indices.as_ptr();
    } else {
        swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        swapchain_create_info.queue_family_index_count = 0;
        swapchain_create_info.p_queue_family_indices = ptr::null();
    }
    swapchain_create_info.pre_transform = 
    swapchain_support
        .capabilities
        .unwrap()
        .current_transform;
    swapchain_create_info.composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
    swapchain_create_info.present_mode = present_mode;
    swapchain_create_info.clipped = vk::TRUE;
    swapchain_create_info.old_swapchain = vk::SwapchainKHR::null();
    
    let swapchain_loader = khr::Swapchain::new(&instance, &device);
    let swapchain = unsafe {
        println!("check");
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create swap chain")
    };
    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Failed to get swap chain images")
    };

    event_loop.run(move |event, _, control_flow| {
        // handle event
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    println!("CloseRequested");
                    // unsafe {
                    //     core.device
                    //         .device_wait_idle()
                    //         .expect("Failed to wait device idle.");
                    // }
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                winit::event::WindowEvent::KeyboardInput { input, .. } => match input {
                    winit::event::KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (
                            Some(winit::event::VirtualKeyCode::Escape),
                            winit::event::ElementState::Pressed,
                        ) => {
                            dbg!();

                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            winit::event::Event::MainEventsCleared => {
                //core.window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_window_id) => {
                // core.burn_frame();
            }
            _ => (),
        }
    });
}
