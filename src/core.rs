use winit;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use ash::Entry;

use crate::debug::{destroy_debug_messenger, setup_debug_messenger};
use crate::instance::create_instance;
use crate::logical_device::create_logical_device;
use crate::physical_device::pick_pyhsical_device;
use crate::queue;
use crate::surface::create_surface;
use crate::swapchain::create_swap_chain;
use crate::pipeline::Pipeline;
pub struct Core {
    window: winit::window::Window,
    entry: Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
    debug_utils_messenger: Option<vk::DebugUtilsMessengerEXT>,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swap_chain: vk::SwapchainKHR,
    swap_chain_loader: ash::extensions::khr::Swapchain,
    swap_chain_images: Vec<vk::Image>,
    swap_chain_image_format: vk::Format,
    swap_chain_extent: vk::Extent2D,
    swap_chain_image_views: Vec<vk::ImageView>,
    pipeline: Pipeline,
}

impl Core {
    pub fn build(
        window_name: &str,
        event_loop: winit::event_loop::EventLoop<()>,
    ) -> (Self, winit::event_loop::EventLoop<()>) {
        // start a window

        let window = winit::window::WindowBuilder::new()
            .with_title(window_name)
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop)
            .unwrap();

        let entry = Entry::new().unwrap();
        let instance = create_instance(&entry);
        let surface = unsafe { create_surface(&entry, &instance, &window).expect("") };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let physical_device = pick_pyhsical_device(&instance, &surface);
        let device = create_logical_device(&instance, &physical_device, &surface);
        let indicies = queue::find_queue_families(&instance, &physical_device, &surface);
        let graphics_queue = match indicies.graphics_family {
            Some(graphics_family) => unsafe { 
                device.get_device_queue(graphics_family, 0) },
            None => {
                panic!("Failed to find a suitable queue family");
            }
        };
        let present_queue = match indicies.present_family {
            Some(present_family) => unsafe { device.get_device_queue(present_family, 0) },
            None => {
                panic!("Failed to find a suitable queue family");
            }
        };
        let debug_utils_messenger = setup_debug_messenger(&entry, &instance);
        let (
            swap_chain,
            swap_chain_loader,
            swap_chain_images,
            swap_chain_image_format,
            swap_chain_extent,
            swap_chain_image_views,
        ) = create_swap_chain(&instance, &device, &physical_device, &surface, &window);
        let pipeline = Pipeline::new(
            "src/shaders/simple_shader.vert.spv",
            "src/shaders/simple_shader.frag.spv",
        );
        pipeline.create_graphic_pipeline();
        (
            Core {
                window,
                entry,
                instance,
                surface,
                surface_loader,
                debug_utils_messenger,
                physical_device,
                device,
                graphics_queue,
                present_queue,
                swap_chain,
                swap_chain_loader,
                swap_chain_images,
                swap_chain_image_format,
                swap_chain_extent,
                swap_chain_image_views,
                pipeline,
            },
            event_loop,
        )
    }

    fn burn_frame(&self) {}

    pub fn burn((core, event_loop): (Self, winit::event_loop::EventLoop<()>)) {
        event_loop.run(move |event, _, control_flow| {
            // handle event
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
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
                    core.window.request_redraw();
                }
                winit::event::Event::RedrawRequested(_window_id) => {
                    core.burn_frame();
                }
                _ => (),
            }
        });
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            for view in self.swap_chain_image_views.iter() {
                self.device.destroy_image_view(*view, None);
            }
            self.swap_chain_loader
                .destroy_swapchain(self.swap_chain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            destroy_debug_messenger(&self.entry, &self.instance, self.debug_utils_messenger);

            self.instance.destroy_instance(None);
        }
    }
}
