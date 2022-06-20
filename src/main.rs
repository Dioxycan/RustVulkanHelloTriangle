mod core;
mod debug;
use crate::core::Core;

fn main() {
   Core::burn(Core::build("hello",winit::event_loop::EventLoop::new()));
}
