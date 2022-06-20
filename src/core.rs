use winit;

use ash::extensions::{khr,ext};

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk_make_version;
use ash::Entry;

use std::default::Default;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::ptr;

use crate::debug::{check_validation_layer_support, ValidationInfo, VALIDATION};

pub struct Core {
    window: winit::window::Window,
    entry: Entry,
    instance: ash::Instance,
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
        (
            Core {
                window,
                instance: Core::create_instance(&entry),
                entry,
            },
            event_loop,
        )
    }

    fn create_instance(entry: &Entry) -> ash::Instance {
        if VALIDATION.is_enable && check_validation_layer_support(entry) == false {
            panic!("Validation layers requested, but not available!");
        }

        let app_name = CString::new("Hello").unwrap();
        let engine_name = CString::new("Engine").unwrap();
        let app_info = vk::ApplicationInfo {
            application_version: vk_make_version!(1, 0, 0),
            p_application_name: app_name.as_ptr(),
            engine_version: vk_make_version!(1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            api_version: vk_make_version!(1, 0, 0),
            ..Default::default()
        };

        let required_extensions: Vec<*const i8> = vec![
            khr::Surface::name().as_ptr(),
            khr::Win32Surface::name().as_ptr(),
            ext::DebugUtils::name().as_ptr(),
        ];

        let requred_validation_layer_raw_names: Vec<CString> = VALIDATION
            .required_validation_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const i8> = requred_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,

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

            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };
        instance
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
            self.instance.destroy_instance(None);
        }
    }
}
