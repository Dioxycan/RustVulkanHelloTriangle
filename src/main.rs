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
pub const vert_file_path_const :&'static str= "src/shaders/shader.vert.spv";
pub const frag_file_path_const :&'static str= "src/shaders/shader.frag.spv";
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
struct Window {
    window: winit::window::Window,
    width: u32,
    height: u32,
}
impl Window {
    fn new(
        window_name: &'static str,
        event_loop: &winit::event_loop::EventLoop<()>,
        width: u32,
        height: u32,
    ) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title(window_name)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .expect("Failed to create window");
        Window {
            window,
            width,
            height,
        }
    }
    unsafe fn create_surface(
        &self,
        device_builder: &DeviceBuilder,
    ) -> (vk::SurfaceKHR, khr::Surface) {
        let window = &device_builder.window.window;
        let surface_loader = khr::Surface::new(
            device_builder.entry,
            device_builder.instance.as_ref().unwrap(),
        );
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
            device_builder.entry,
            device_builder.instance.as_ref().unwrap(),
        );
        let surface = win32_surface_loader
            .create_win32_surface(&win32_create_info, None)
            .expect("Failed to create win32 surface");
        (surface, surface_loader)
    }
    fn run(&mut self, event_loop: winit::event_loop::EventLoop<()>) {
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
}
pub struct SurfaceStuff {
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
}
impl SurfaceStuff {
    fn new(window: &Window, device_builder: &DeviceBuilder) -> Self {
        let (surface, surface_loader) = unsafe { window.create_surface(device_builder) };
        SurfaceStuff {
            surface: surface,
            surface_loader: surface_loader,
        }
    }
}
struct SwapChainSupportDetails {
    capabilities: Option<vk::SurfaceCapabilitiesKHR>,
    formats: Option<Vec<vk::SurfaceFormatKHR>>,
    present_modes: Option<Vec<vk::PresentModeKHR>>,
}
impl SwapChainSupportDetails {
    fn query_swapchain_support(
        physical_device: &vk::PhysicalDevice,
        surface_stuff: &SurfaceStuff,
    ) -> Self {
        SwapChainSupportDetails {
            capabilities: match unsafe {
                surface_stuff
                    .surface_loader
                    .get_physical_device_surface_capabilities(
                        *physical_device,
                        surface_stuff.surface,
                    )
            } {
                Ok(capabilities) => Some(capabilities),
                Err(err) => {
                    eprintln!("Failed to get surface capabilities{:?}", err);
                    None
                }
            },
            formats: match unsafe {
                surface_stuff
                    .surface_loader
                    .get_physical_device_surface_formats(*physical_device, surface_stuff.surface)
            } {
                Ok(formats) => Some(formats),
                Err(err) => {
                    eprintln!("Failed to get surface formats{:?}", err);
                    None
                }
            },
            present_modes: match unsafe {
                surface_stuff
                    .surface_loader
                    .get_physical_device_surface_present_modes(
                        *physical_device,
                        surface_stuff.surface,
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
struct DeviceBuilder<'a> {
    entry: &'a ash::Entry,
    window: &'a Window,
    instance: Option<ash::Instance>,
    device: Option<ash::Device>,
    physical_device: Option<vk::PhysicalDevice>,
    required_extensions: Vec<*const i8>,
    surface_stuff: Option<SurfaceStuff>,
    command_pool: Option<vk::CommandPool>,
}
impl<'a> DeviceBuilder<'a> {
    pub fn new(entry: &'a ash::Entry, window: &'a Window) -> DeviceBuilder<'a> {
        DeviceBuilder {
            entry,
            window,
            instance: None,
            device: None,
            physical_device: None,
            required_extensions: Vec::new(),
            surface_stuff: None,
            command_pool: None,
        }
    }

    fn create_required_extesions(&mut self) {
        self.required_extensions = vec![
            khr::Surface::name().as_ptr(),
            khr::Win32Surface::name().as_ptr(),
            ext::DebugUtils::name().as_ptr(),
        ];
    }
    fn setup_debug_messenger(&mut self) -> (ext::DebugUtils, Option<vk::DebugUtilsMessengerEXT>) {
        let debug_utils_loader = ext::DebugUtils::new(self.entry, self.instance.as_ref().unwrap());
        if VALIDATION.is_enable {
            let create_info = populate_debug_messenger_create_info();
            let debug_utils_messenger = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(&create_info, None)
                    .expect("Debug Utils Callback not found")
            };
            (debug_utils_loader, Some(debug_utils_messenger))
        } else {
            (debug_utils_loader, None)
        }
    }
    fn create_instance(&mut self) {
        if VALIDATION.is_enable && check_validation_layer_support(self.entry) == false {
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

            enabled_extension_count: self.required_extensions.len() as u32,
            pp_enabled_extension_names: self.required_extensions.as_ptr(),

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
                &populate_debug_messenger_create_info()
                    as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
            } else {
                ptr::null()
            },
            ..Default::default()
        };

        self.instance = Some(unsafe {
            self.entry
                .create_instance(&instance_create_info, None)
                .expect("Failed to create instance")
        });
    }
    fn pick_physical_device(&mut self) {
        let physical_devices = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };
        for physical_device in physical_devices.iter() {
            self.physical_device = Some(*physical_device);
            unsafe {
                let properties = self.instance
                    .as_ref()
                    .unwrap()
                    .get_physical_device_properties(*physical_device);
                println!(
                    "Physical Device: {}",
                    CStr::from_ptr(properties.device_name.as_ptr()).to_str().unwrap()
                );
            }
            if self.is_device_suitable() {
                return;
            }
        }
        panic!("Failed to find a suitable GPU!");
    }
    fn is_device_suitable(&mut self) -> bool {
        let indices = self.find_queue_families();
        let extensions_supported = self.check_device_extension_support();
        let swapchain_adequate = SwapChainSupportDetails::query_swapchain_support(
            &self.physical_device.unwrap(),
            &self.surface_stuff.as_ref().unwrap(),
        );
        let supported_features = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .get_physical_device_features(self.physical_device.unwrap())
        };
        
        indices.is_complete()
            && extensions_supported
            && swapchain_adequate.is_Suitable()
            && supported_features.sampler_anisotropy as u32 == 1
    }
    fn find_queue_families(&mut self) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();
        let queue_families = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .get_physical_device_queue_family_properties(self.physical_device.unwrap())
        };
        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                indices.graphics_family = Some(i as u32);
            }
            let is_present_support = unsafe {
                self.surface_stuff
                    .as_ref()
                    .unwrap()
                    .surface_loader
                    .get_physical_device_surface_support(
                        self.physical_device.unwrap(),
                        i as u32,
                        self.surface_stuff.as_ref().unwrap().surface,
                    )
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
    fn check_device_extension_support(&mut self) -> bool {
        let available_extensions = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .enumerate_device_extension_properties(self.physical_device.unwrap())
        };

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
    fn create_logical_device(&mut self) -> (vk::Queue, vk::Queue) {
        let indices = self.find_queue_families();
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
        self.device = Some(unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .create_device(self.physical_device.unwrap(), &create_info, None)
                .expect("Failed to create logical device")
        });

        unsafe {
            (
                self.device
                    .as_ref()
                    .unwrap()
                    .get_device_queue(indices.graphics_family.unwrap(), 0),
                self.device
                    .as_ref()
                    .unwrap()
                    .get_device_queue(indices.present_family.unwrap(), 0),
            )
        }
    }
    fn create_command_pool(&mut self) {
        let queue_family_indices = self.find_queue_families();
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_family.unwrap())
            .build();
        self.command_pool = Some(unsafe {
            self.device
                .as_ref()
                .unwrap()
                .create_command_pool(&pool_info, None)
                .expect("Failed to create command pool")
        });
    }
    fn build(mut self) -> Device {
        self.create_required_extesions();

        self.create_instance();
        self.surface_stuff = Some(SurfaceStuff::new(self.window, &self));
        self.pick_physical_device();
        let (graphics_queue, present_queue) = self.create_logical_device();
        self.create_command_pool();
        let (debug_utils_loader, debug_messenger) = self.setup_debug_messenger();
        let queue_family_indices = self.find_queue_families();

        Device {
            swapchain_support: SwapChainSupportDetails::query_swapchain_support(
                self.physical_device.as_ref().unwrap(),
                self.surface_stuff.as_ref().unwrap(),
            ),
            instance: self.instance.unwrap(),
            device: self.device.unwrap(),
            physical_device: self.physical_device.unwrap(),
            required_extensions: self.required_extensions,
            debug_messenger,
            debug_utils_loader,
            surface_stuff: self.surface_stuff.unwrap(),
            command_pool: self.command_pool.unwrap(),
            graphics_queue,
            present_queue,
            queue_family_indices,
        }
    }
}
struct Device {
    instance: ash::Instance,
    device: ash::Device,
    required_extensions: Vec<*const i8>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    surface_stuff: SurfaceStuff,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_pool: vk::CommandPool,
    queue_family_indices: QueueFamilyIndices,
    swapchain_support: SwapChainSupportDetails,
}

impl<'a> Device {
    fn builder(entry: &'a ash::Entry, window: &'a Window) -> DeviceBuilder<'a> {
        DeviceBuilder::new(entry, window)
    }
    fn find_supported_format(
        &self,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in formats {
            let props = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device, *format)
            };
            if (tiling == vk::ImageTiling::LINEAR
                && props.linear_tiling_features.contains(features))
                || props.optimal_tiling_features.contains(features)
                || props.buffer_features.contains(features)
            {
                return *format;
            }
        }
        panic!("Failed to find supported format");
    }
    fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        let mem_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        for i in 0..mem_properties.memory_type_count {
            if type_filter & (1 << i) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return i;
            }
        }
        panic!("Failed to find suitable memory type");
    }
    fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .expect("Failed to create buffer")
        };
        let buffer_memory_requirements =
            unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let memory_type_index =
            self.find_memory_type(buffer_memory_requirements.memory_type_bits, properties);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();
        let buffer_memory = unsafe {
            self.device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate buffer memory")
        };
        unsafe {
            self.device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind buffer memory")
        };
        (buffer, buffer_memory)
    }
    fn begin_single_time_commands(&self) -> vk::CommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .command_pool(self.command_pool)
            .build();
        let command_buffer = unsafe {
            self.device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffer")[0]
        };
        let begin_info = vk::CommandBufferBeginInfo::builder().build();
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin command buffer")
        };
        command_buffer
    }
    fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer")
        };
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .build();
        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], vk::Fence::null())
                .expect("Failed to submit command buffer")
        };
        unsafe {
            self.device
                .queue_wait_idle(self.graphics_queue)
                .expect("Failed to wait for command buffer")
        };
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &[command_buffer])
        };
    }
    fn copy_buffer(&self, src_buffer: vk::Buffer, dst_buffer: vk::Buffer, size: vk::DeviceSize) {
        let copy_info = vk::BufferCopy::builder().size(size).build();
        let command_buffer = self.begin_single_time_commands();
        unsafe {
            self.device
                .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_info])
        };
        self.end_single_time_commands(command_buffer);
    }
    fn copy_buffer_to_image(
        &self,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
        layer_count: u32,
    ) {
        let buffer_image_copy = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(layer_count)
                    .build(),
            )
            .image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
            .image_extent(
                vk::Extent3D::builder()
                    .width(width)
                    .height(height)
                    .depth(1)
                    .build(),
            )
            .build();
        let command_buffer = self.begin_single_time_commands();
        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_image_copy],
            )
        };
        self.end_single_time_commands(command_buffer);
    }
    fn create_image_with_info(
        &self,
        image_create_info: vk::ImageCreateInfo,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Image, vk::DeviceMemory) {
        let image = unsafe {
            self.device
                .create_image(&image_create_info, None)
                .expect("Failed to create image")
        };
        let memory_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let memory_type_index =
            self.find_memory_type(memory_requirements.memory_type_bits, properties);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();
        let image_memory = unsafe {
            self.device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate image memory")
        };
        unsafe {
            self.device
                .bind_image_memory(image, image_memory, 0)
                .expect("Failed to bind image memory")
        };
        (image, image_memory)
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            match self.debug_messenger {
                Some(debug_messenger) => {
                    self.debug_utils_loader
                        .destroy_debug_utils_messenger(debug_messenger, None);
                }
                None => {}
            }
            self.surface_stuff
                .surface_loader
                .destroy_surface(self.surface_stuff.surface, None);
            self.instance.destroy_instance(None);
        };
    }
}
struct PipelineConfigInfo {
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    multisample_info: vk::PipelineMultisampleStateCreateInfo,
    color_blend_info: vk::PipelineColorBlendStateCreateInfo,
    depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    subpass: u32,
}
impl PipelineConfigInfo {
    fn default_pipeline_config_info(width: f32, height: f32) -> Self {
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 800.0,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: width as u32,
                height: height as u32,
            },
        };

        // let viewport_info = vk::PipelineViewportStateCreateInfo{
        //     s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        //     p_next: ptr::null(),
        //     flags: Default::default(),
        //     viewport_count: 1,
        //     p_viewports: &viewport,
        //     scissor_count: 1,
        //     p_scissors: &scissor,
        // };

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .build();
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();
        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&[color_blend_attachment])
            .build();
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
            .build();
        PipelineConfigInfo {
            viewport,
            scissor,
            input_assembly_info,
            rasterization_info,
            multisample_info,
            color_blend_info,
            depth_stencil_info,
            pipeline_layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            subpass: 0,
        }
    }
}
struct Pipeline {
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
    pipeline_config_info: PipelineConfigInfo,
    device: Arc<Device>,
    graphic_pipeline: vk::Pipeline,
}
impl Pipeline {
    fn new(device: Arc<Device>,pipeline_config_info:PipelineConfigInfo) -> Self {
        Pipeline {
            vert_shader_module: vk::ShaderModule::null(),
            frag_shader_module: vk::ShaderModule::null(),
            pipeline_config_info,
            device,
            graphic_pipeline: vk::Pipeline::null(),
        }
    }
    fn null(device:Arc<Device>)->Self{
        Pipeline{
            vert_shader_module: vk::ShaderModule::null(),
            frag_shader_module: vk::ShaderModule::null(),
            pipeline_config_info: PipelineConfigInfo::default_pipeline_config_info(0.0,0.0),
            device,
            graphic_pipeline: vk::Pipeline::null(),
        }
    }
    fn build(mut self, vert_file_path: &'static str, frag_file_path: &'static str) -> Self {
        let vert_code = Pipeline::read_shader(vert_file_path);
        let frag_code = Pipeline::read_shader(frag_file_path);
        self.vert_shader_module = self.create_shader_module(vert_code);
        self.frag_shader_module = self.create_shader_module(frag_code);
        // let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        //     module: self.vert_shader_module,
        //     p_name: b"main\0".as_ptr() as *const _,
        //     stage: vk::ShaderStageFlags::VERTEX,
        //     ..Default::default()
        // };
        // let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        //     module: self.frag_shader_module,
        //     p_name: b"main\0".as_ptr() as *const _,
        //     stage: vk::ShaderStageFlags::FRAGMENT,
        //     ..Default::default()
        // };
        // let shader_stages =
        // [vert_shader_stage_info, frag_shader_stage_info];
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(self.vert_shader_module)
                .name(unsafe { CStr::from_ptr(b"main\0".as_ptr() as *const _) })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(self.frag_shader_module)
                .name(unsafe { CStr::from_ptr(b"main\0".as_ptr() as *const _) })
                .build(),
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[])
            .build();
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(&[self.pipeline_config_info.viewport])
            .scissor_count(1)
            .scissors(&[self.pipeline_config_info.scissor])
            .build();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &self.pipeline_config_info.input_assembly_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_info,
            p_rasterization_state: &self.pipeline_config_info.rasterization_info,
            p_multisample_state: &self.pipeline_config_info.multisample_info,
            p_depth_stencil_state: &self.pipeline_config_info.depth_stencil_info,
            p_color_blend_state: &self.pipeline_config_info.color_blend_info,
            p_dynamic_state: ptr::null(),
            layout: self.pipeline_config_info.pipeline_layout,
            render_pass: self.pipeline_config_info.render_pass,
            subpass: self.pipeline_config_info.subpass,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
        };
        self.graphic_pipeline = unsafe {
            self.device
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .expect("failed to create graphics pipeline")[0]
        };
        self
    }

    fn read_shader(filename: &str) -> Vec<u8> {
        let mut file = File::open(filename).expect("Failed to open shader file");
        let mut contents: Vec<u8> = vec![];

        file.read_to_end(&mut contents)
            .expect("Failed to read shader file");
        contents
    }
    fn create_shader_module(&self, code: Vec<u8>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        };
        unsafe {
            self.device
                .device
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create shader module")
        }
    }
}
impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_shader_module(self.vert_shader_module, None);
            self.device
                .device
                .destroy_shader_module(self.frag_shader_module, None);
            self.device
                .device
                .destroy_pipeline(self.graphic_pipeline, None);
        }
    }
}
struct SwapChain {
    device: Arc<Device>,
    window_extent: vk::Extent2D,
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_frame_buffers: Vec<vk::Framebuffer>,
    render_pass: vk::RenderPass,
    depth_images: Vec<vk::Image>,
    depth_image_views: Vec<vk::ImageView>,
    depth_image_memory: Vec<vk::DeviceMemory>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    image_in_flight: Vec<vk::Fence>,
    current_frame: usize,
}

impl SwapChain {
    fn new(device: Arc<Device>, extent: vk::Extent2D,swapchain_loader:khr::Swapchain) -> Self {
        SwapChain {
            swapchain_loader,
            device,
            window_extent: extent,
            swapchain: vk::SwapchainKHR::null(),
            swapchain_images: vec![],
            swapchain_image_views: vec![],
            swapchain_format: vk::Format::UNDEFINED,
            swapchain_extent: extent,
            swapchain_frame_buffers: vec![],
            render_pass: vk::RenderPass::null(),
            depth_images: vec![],
            depth_image_views: vec![],
            depth_image_memory: vec![],
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
            image_in_flight: vec![],
            current_frame: 0,
        }
    }
    fn build(&mut self,window: &winit::window::Window) {
        self.create_swapchain(window);
        self.create_image_views();
        self.create_render_pass();
        self.create_depth_resources();
        self.create_framebuffers();
        self.create_sync_objects();
    }

    fn create_swapchain(&mut self,window: &winit::window::Window) {
        let surface_format = self.choose_swap_surface_format();
        let present_mode = self.choose_swap_present_mode();
        let extent = self.choose_swap_extent();
        let mut image_count = self
            .device
            .swapchain_support
            .capabilities
            .unwrap()
            .min_image_count
            + 1;
        if self
            .device
            .swapchain_support
            .capabilities
            .unwrap()
            .max_image_count
            > 0
            && image_count
                > self
                    .device
                    .swapchain_support
                    .capabilities
                    .unwrap()
                    .max_image_count
        {
            image_count = self
                .device
                .swapchain_support
                .capabilities
                .unwrap()
                .max_image_count;
        }
        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: self.device.surface_stuff.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ..Default::default()
        };
        let queue_family_indices = &self.device.queue_family_indices;
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
        swapchain_create_info.pre_transform = self
            .device
            .swapchain_support
            .capabilities
            .unwrap()
            .current_transform;
        swapchain_create_info.composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        swapchain_create_info.present_mode = present_mode;
        swapchain_create_info.clipped = vk::TRUE;
        swapchain_create_info.old_swapchain = vk::SwapchainKHR::null();
        self.swapchain = unsafe {
            println!("check");
            self.swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swap chain")
        };
        let swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };
        self.swapchain_images = unsafe {
            self.swapchain_loader
                .get_swapchain_images(self.swapchain)
                .expect("Failed to get swap chain images")
        };
        self.swapchain_format = surface_format.format;
        self.swapchain_extent = extent;
    }

    fn choose_swap_surface_format(&self) -> vk::SurfaceFormatKHR {
        match &self.device.swapchain_support.formats {
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
    fn choose_swap_present_mode(&self) -> vk::PresentModeKHR {
        match &self.device.swapchain_support.present_modes {
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
    fn choose_swap_extent(&mut self) -> vk::Extent2D {
        match self.device.swapchain_support.capabilities {
            Some(capabilities) => {
                if capabilities.current_extent.width != std::u32::MAX {
                    return capabilities.current_extent;
                } else {
                    let mut actual_extent = self.window_extent;
                    actual_extent.width = std::cmp::max(
                        capabilities.min_image_extent.width,
                        std::cmp::min(capabilities.max_image_extent.width, actual_extent.width),
                    );
                    actual_extent.height = std::cmp::max(
                        capabilities.min_image_extent.height,
                        std::cmp::min(capabilities.max_image_extent.height, actual_extent.height),
                    );
                    return actual_extent;
                }
            }
            _ => {
                panic!("failed to choose swap extent");
            }
        }
    }
    fn find_depth_format(&self) -> vk::Format {
        self.device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }
    fn create_image_views(&mut self) {
        self.swapchain_image_views = Vec::with_capacity(self.swapchain_images.len());
        for image_index in self.swapchain_images.iter() {
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image: *image_index,
                view_type: vk::ImageViewType::TYPE_2D,
                format: self.swapchain_format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            let image_view = unsafe {
                self.device
                    .device
                    .create_image_view(&image_view_create_info, None)
                    .expect("Failed to create image view")
            };
            self.swapchain_image_views.push(image_view);
        }
    }
    fn create_depth_resources(&mut self) {
        let depth_format = self.find_depth_format();
        self.depth_images = Vec::with_capacity(self.swapchain_images.len());
        self.depth_image_views = Vec::with_capacity(self.swapchain_images.len());
        self.depth_image_memory = Vec::with_capacity(self.swapchain_images.len());
        for _i in 0..self.depth_images.len() {
            let depth_image_create_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageCreateFlags::empty(),
                image_type: vk::ImageType::TYPE_2D,
                extent: vk::Extent3D {
                    width: self.swapchain_extent.width,
                    height: self.swapchain_extent.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                format: depth_format,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                samples: vk::SampleCountFlags::TYPE_1,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            let (depth_image, depth_memory) = unsafe {
                self.device.create_image_with_info(
                    depth_image_create_info,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )
            };
            self.depth_images.push(depth_image);
            self.depth_image_memory.push(depth_memory);
            let image_view_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                image: depth_image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: depth_format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            let depth_image_view = unsafe {
                self.device
                    .device
                    .create_image_view(&image_view_create_info, None)
                    .expect("Failed to create texture image view")
            };
            self.depth_image_views.push(depth_image_view);
        }
    }
    fn create_render_pass(&mut self) {
        let depth_attachment = vk::AttachmentDescription {
            format: self.find_depth_format(),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let color_attachment = vk::AttachmentDescription {
            format: self.swapchain_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };
        let subpass_description = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            ..Default::default()
        };
        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
            ..Default::default()
        };
        let attachments = [color_attachment, depth_attachment];
        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass_description,
            dependency_count: 1,
            p_dependencies: &subpass_dependency,
            ..Default::default()
        };
        self.render_pass = unsafe {
            self.device
                .device
                .create_render_pass(&render_pass_create_info, None)
                .expect("Failed to create render pass")
        };
    }
    fn create_framebuffers(&mut self) {
        self.swapchain_frame_buffers
            .reserve(self.swapchain_images.len());
        for i in 0..self.swapchain_images.len() {
            let attachments = [self.swapchain_image_views[i], self.depth_image_views[i]];
            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: self.render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.swapchain_extent.width,
                height: self.swapchain_extent.height,
                layers: 1,
                ..Default::default()
            };
            let framebuffer = unsafe {
                self.device
                    .device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create framebuffer")
            };
            self.swapchain_frame_buffers.push(framebuffer);
        }
    }
    fn create_sync_objects(&mut self) {
        self.image_available_semaphores
            .reserve(MAX_FRAMES_IN_FLIGHT);
        self.render_finished_semaphores
            .reserve(MAX_FRAMES_IN_FLIGHT);
        self.in_flight_fences.reserve(MAX_FRAMES_IN_FLIGHT);
        self.image_in_flight.reserve(self.swapchain_images.len());
        let image_available_semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            ..Default::default()
        };
        let render_finished_semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            ..Default::default()
        };
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            self.image_available_semaphores.push(unsafe {
                self.device
                    .device
                    .create_semaphore(&image_available_semaphore_create_info, None)
                    .expect("Failed to create image available semaphore")
            });
            self.render_finished_semaphores.push(unsafe {
                self.device
                    .device
                    .create_semaphore(&render_finished_semaphore_create_info, None)
                    .expect("Failed to create render finished semaphore")
            });
            self.in_flight_fences.push(unsafe {
                self.device
                    .device
                    .create_fence(
                        &vk::FenceCreateInfo {
                            s_type: vk::StructureType::FENCE_CREATE_INFO,
                            p_next: ptr::null(),
                            flags: vk::FenceCreateFlags::SIGNALED,
                            ..Default::default()
                        },
                        None,
                    )
                    .expect("Failed to create fence")
            });
        }
    }
    fn acquire_next_image(&mut self) -> Result<(u32, bool), vk::Result> {
        unsafe {
            self.device
                .device
                .wait_for_fences(&self.in_flight_fences, true, std::u64::MAX)
                .expect("Failed to wait for fences")
        };
        unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        }
    }
    fn submit_command_buffer(
        &mut self,
        command_buffer: &vk::CommandBuffer,
        image_index: usize,
    ) -> Result<bool, vk::Result> {
        if (self.image_in_flight[image_index] != vk::Fence::null()) {
            unsafe {
                self.device
                    .device
                    .wait_for_fences(&[self.image_in_flight[image_index]], true, std::u64::MAX)
                    .expect("Failed to wait for fence")
            };
        }
        self.image_in_flight[image_index] = self.in_flight_fences[self.current_frame];
        let wait_semaphores = self.image_available_semaphores[self.current_frame];
        let wait_dst_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        let signal_semaphores = self.render_finished_semaphores[self.current_frame];
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &wait_semaphores,
            p_wait_dst_stage_mask: &wait_dst_stage_mask,
            command_buffer_count: 1,
            p_command_buffers: command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: &signal_semaphores,
            ..Default::default()
        };
        unsafe {
            self.device
                .device
                .reset_fences(&[self.in_flight_fences[self.current_frame]])
                .expect("Failed to reset fence");
            self.device
                .device
                .queue_submit(
                    self.device.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit command buffer")
        };
        let swapchains = [self.swapchain];
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &signal_semaphores,
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: image_index as *const u32,
            p_results: ptr::null_mut(),
            ..Default::default()
        };
        unsafe {
            self.swapchain_loader
                .queue_present(self.device.present_queue, &present_info)
        }
    }
    fn extent_aspect_ratio(&self) -> f32 {
        self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32
    }

}
impl Drop for SwapChain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_image_views.iter().for_each(|view| {
                self.device.device.destroy_image_view(*view, None);
            });
            if (self.swapchain != vk::SwapchainKHR::null()) {
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);
            }
            self.depth_images.iter().for_each(|image| {
                self.device.device.destroy_image(*image, None);
            });
            self.depth_image_views.iter().for_each(|image_view| {
                self.device.device.destroy_image_view(*image_view, None);
            });
            self.depth_image_memory.iter().for_each(|memory| {
                self.device.device.free_memory(*memory, None);
            });
            self.swapchain_frame_buffers
                .iter()
                .for_each(|frame_buffer| {
                    self.device.device.destroy_framebuffer(*frame_buffer, None);
                });
            self.device
                .device
                .destroy_render_pass(self.render_pass, None);
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device
                    .device
                    .destroy_fence(self.in_flight_fences[i], None);
            }
        }
    }
}
struct Core {
    entry: ash::Entry,
    window: Window,
    pipeline: Option<Arc<Pipeline>>,
    device: Arc<Device>,
    swapchain: SwapChain,
    pipeline_layout: vk::PipelineLayout,
    command_buffers: Vec<vk::CommandBuffer>,
}
impl Core {
    fn new(event_loop: &winit::event_loop::EventLoop<()>, width: u32, height: u32) -> Self {
        let entry = ash::Entry::new().expect("Failed to create entry");
        let window = Window::new("hello", &event_loop, width, height);

        let device: Arc<Device> = Arc::new(Device::builder(&entry, &window).build());
        let mut pipeline=None;
        let swapchain_loader = khr::Swapchain::new(&device.instance.clone(), &device.device.clone());
        let mut swapchain = SwapChain::new(
            device.clone(),
            vk::Extent2D {
                width: window.width,
                height: window.height,
            },
            swapchain_loader,
        );
       swapchain.build(&window.window);
        Core {
            entry,
            window,
            device: device.clone(),
            pipeline,
            swapchain,
            pipeline_layout: vk::PipelineLayout::null(),
            command_buffers: vec![],
        }
    }
    fn build(mut self)->Self{
        self.create_pipeline_layout();
        self.create_pipeline();
        self
    }
    fn create_pipeline_layout(&mut self) {
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };
        let pipeline_layout = unsafe {
            self.device
                .device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Failed to create pipeline layout")
        };
        self.pipeline_layout = pipeline_layout;
    }
    fn create_pipeline(&mut self) {
        let mut pipeline_config_info  = PipelineConfigInfo::default_pipeline_config_info(
            self.swapchain.swapchain_extent.width as f32,
            self.swapchain.swapchain_extent.height as f32,
        );
        pipeline_config_info.render_pass =  self.swapchain.render_pass;
        pipeline_config_info.pipeline_layout =self.pipeline_layout;
        self.pipeline = Some(Arc::new(
            Pipeline::new(self.device.clone(),pipeline_config_info).build(vert_file_path_const, frag_file_path_const),
        ));
    }
    fn create_command_buffers() {}
    fn draw_frame() {}
    fn run(&mut self, event_loop: winit::event_loop::EventLoop<()>) {
        self.window.run(event_loop);
    }
}
impl Drop for Core{
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    Core::new(&event_loop, 800, 600).build().run(event_loop);
}
