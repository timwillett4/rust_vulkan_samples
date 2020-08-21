use crate::logger;
use crate::graphics;

use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;

pub enum UpdateFrequency {
    /// appropriate for GUI apps
    OnEvent,
    /// appropriate for game/graphics apps
    Continuous,
}

pub struct App{
    pub update_frequency: UpdateFrequency,
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
    event_loop: EventLoop<()>,
}

impl App{

    pub fn new(update_frequency: UpdateFrequency) -> App {

        // @TODO - add configuration options for logger
        logger::init();

        // @TODO add proper error handling (ie return result instead of panic)
        let instance =  {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan Instance")
        };

        let event_loop = EventLoop::new();
        // @TODO - proper error handling on result
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();


        App {
            update_frequency,
            instance,
            surface,
            event_loop,
        }
    }

    /// Blocks until Application is complete
    pub fn run(&self) {

        let update_action = match self.update_frequency {
               UpdateFrequency::OnEvent => ControlFlow::Wait,
               UpdateFrequency::Continuous => ControlFlow::Poll,
            };

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


