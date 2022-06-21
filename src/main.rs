mod core;
mod debug;
mod physical_device;
mod queue;
mod logical_device;

use crate::core::Core;

fn main() {
   Core::burn(Core::build("hello",winit::event_loop::EventLoop::new()));
}
