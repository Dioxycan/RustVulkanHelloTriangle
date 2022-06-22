mod core;
mod instance;
mod debug;
mod physical_device;
mod queue;
mod logical_device; 
mod surface;
mod swapchain;
mod pipeline;
use crate::core::Core;

fn main() {
   Core::burn(Core::build("hello",winit::event_loop::EventLoop::new()));
}
