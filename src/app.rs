use crate::logger;
use crate::graphics;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub enum UpdateFrequency {
    /// appropriate for GUI apps
    OnEvent,
    /// appropriate for game/graphics apps
    Continuous,
}

pub struct App{
    pub update_frequency : UpdateFrequency
}

impl App{

    /// Blocks until Application is complete
    pub fn run(&self) {

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let update_action = match self.update_frequency {
               UpdateFrequency::OnEvent => ControlFlow::Wait,
               UpdateFrequency::Continuous => ControlFlow::Poll,
            };

        logger::init();
        //graphics::init();

        event_loop.run(move |event, _, control_flow| {

            *control_flow = update_action;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}


