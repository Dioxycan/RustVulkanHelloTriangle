use winit;
pub struct Core{
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
}

impl Core {
    pub fn build(window_name:&str)->Self{
        // start a window
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(window_name)
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop)
            .unwrap();

        Core{
            event_loop,
            window,
        }
    }
    pub fn burn(self){
        self.event_loop.run(move |event,_, control_flow|{
            // handle event
            match event{
                winit::event::Event::WindowEvent{event,..}=>{
                    match event{
                        winit::event::WindowEvent::CloseRequested=>{
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        },
                        winit::event::WindowEvent::KeyboardInput {input,..}=>{
                            match input{
                                winit::event::KeyboardInput { virtual_keycode,  state ,.. } => {
                                    match(virtual_keycode, state){
                                        (Some(winit::event::VirtualKeyCode::Escape), winit::event::ElementState::Pressed)=>{
                                            dbg!();
                                            *control_flow = winit::event_loop::ControlFlow::Exit;
                                        },
                                        _ => {}
                                    }
                                },
                            }
                        }
                        _=>{}
                    }
                },
                _=>{}
            }
        });
    }
}
