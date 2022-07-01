use ash::version::DeviceV1_0;
use winit;

use crate::command::{create_command_buffers, create_command_pool};
use crate::debug::{destroy_debug_messenger, setup_debug_messenger};
use crate::framebuffer::create_framebuffer;
use crate::instance::create_instance;
use crate::logical_device::create_logical_device;
use crate::physical_device::pick_pyhsical_device;
use crate::pipeline::{create_graphic_pipeline, create_render_pass};
use crate::queue;
use crate::surface::create_surface;
use crate::swapchain::{create_image_views, create_swapchain};
use ash::version::InstanceV1_0;
use ash::vk;
use ash::Entry;
use std::ptr;
const MAX_FRAMES_IN_FLIGHT: usize = 2;
struct SyncObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,
}
// pub struct Core{
//     window:winit::window::Window,
//     entry:ash::Entry,
//     instance:ash::Instance,
// }

pub struct Core {
    window: winit::window::Window,
    entry: Entry,
    instance: ash::Instance,
    surface: ash::vk::SurfaceKHR,
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
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    frame_buffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffer: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    image_in_flight: Vec<vk::Fence>,
    current_frame: usize,
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

        let entry = ash::Entry::new().unwrap();
        let instance = create_instance(&entry);
        let surface = unsafe { create_surface(&entry, &instance, &window, 1024, 768) };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let physical_device = pick_pyhsical_device(&instance, &surface);
        let device = create_logical_device(&instance, &physical_device, &surface.surface);
        let indicies = queue::find_queue_families(&instance, &physical_device, &surface.surface);
        let graphics_queue = match indicies.graphics_family {
            Some(graphics_family) => unsafe { device.get_device_queue(graphics_family, 0) },
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
        let swapchain_stuff = create_swapchain(
            &instance,
            &device,
            physical_device,
            &window,
            &surface,
            &indicies,
        );
        let render_pass = create_render_pass(&device, swapchain_stuff.swapchain_format);
        let (pipeline, pipeline_layout) =
            create_graphic_pipeline(&device, &swapchain_stuff.swapchain_extent, &render_pass);
        let swapchain_image_views = create_image_views(
            &device,
            swapchain_stuff.swapchain_format,
            &swapchain_stuff.swapchain_images,
        );
        let frame_buffers = create_framebuffer(
            &device,
            &swapchain_image_views,
            &render_pass,
            &swapchain_stuff.swapchain_extent,
        );
        let command_pool =
            create_command_pool(&instance, &physical_device, &device, &surface.surface);
        let command_buffer = create_command_buffers(
            &device,
            &command_pool,
            &pipeline,
            &frame_buffers,
            &render_pass,
            &swapchain_stuff.swapchain_extent,
        );
        let sync_ojbects = Core::create_sync_objects(&device);

        (
            Core {
                image_available_semaphores: sync_ojbects.image_available_semaphores,
                render_finished_semaphores: sync_ojbects.render_finished_semaphores,
                in_flight_fences: sync_ojbects.inflight_fences,
                image_in_flight: vec![vk::Fence::null(); swapchain_stuff.swapchain_images.len()],
                window,
                entry,
                instance,
                surface: surface.surface,
                surface_loader: surface.surface_loader,
                debug_utils_messenger,
                physical_device,
                device,
                graphics_queue,
                present_queue,
                swap_chain: swapchain_stuff.swapchain,
                swap_chain_loader: swapchain_stuff.swapchain_loader,
                swap_chain_images: swapchain_stuff.swapchain_images,
                swap_chain_image_format: swapchain_stuff.swapchain_format,
                swap_chain_extent: swapchain_stuff.swapchain_extent,
                swap_chain_image_views: swapchain_image_views,
                pipeline,
                pipeline_layout,
                render_pass,
                frame_buffers,
                command_buffer,
                command_pool,
                current_frame: 0,
            },
            event_loop,
        )
    }

    fn create_sync_objects(device: &ash::Device) -> SyncObjects {
        let mut sync_objects = SyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            inflight_fences: vec![],
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.inflight_fences.push(inflight_fence);
            }
        }

        sync_objects
    }
    fn burn_frame(&mut self) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
            let ok = self
                .swap_chain_loader
                .acquire_next_image(
                    self.swap_chain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.");
            ok
        };
        if (self.image_in_flight[image_index as usize] != vk::Fence::null()) {
            unsafe {
                self.device
                    .wait_for_fences(&wait_fences, true, std::u64::MAX)
                    .expect("Failed to wait for Fence!");
            }
        }
        self.image_in_flight[image_index as usize] = self.in_flight_fences[self.current_frame];
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            //[TODO] change to command buffer
            p_command_buffers: &self.command_buffer[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        }];
        unsafe {
            self.device
                .reset_fences(&[self.in_flight_fences[self.current_frame]])
                .expect("Failed to reset Fence!");

            let ok = self.device.queue_submit(
                self.graphics_queue,
                &submit_infos,
                self.in_flight_fences[self.current_frame],
            );
        }

        let swapchains = [self.swap_chain];

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            ..Default::default()
        };
        unsafe {
            let ok = self
                .swap_chain_loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn burn((mut core, event_loop): (Self, winit::event_loop::EventLoop<()>)) {
        event_loop.run(move |event, _, control_flow| {
            // handle event
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        unsafe {
                            core.device
                                .device_wait_idle()
                                .expect("Failed to wait device idle.");
                        };
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
                                unsafe {
                                    core.device
                                        .device_wait_idle()
                                        .expect("Failed to wait device idle.");
                                };
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
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            for framebuffer in self.frame_buffers.iter() {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            for view in self.swap_chain_image_views.iter() {
                self.device.destroy_image_view(*view, None);
            }
            self.swap_chain_loader
                .destroy_swapchain(self.swap_chain, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            destroy_debug_messenger(&self.entry, &self.instance, self.debug_utils_messenger);

            self.instance.destroy_instance(None);
        }
    }
}
