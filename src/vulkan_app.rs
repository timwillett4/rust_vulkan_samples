
use crate::logger;

use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId, Window},
};

use vulkano_win::VkSurfaceBuild;

use vulkano::{
    command_buffer::DynamicState,
    device::{Device, Queue},
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::SwapchainImage,
    instance::Instance,
    pipeline::viewport::Viewport,
    swapchain::{Surface, Swapchain},
    sync,
    sync::{GpuFuture},
};


pub struct App{
    vulkan_app: VulkanApp,
    event_handler: Box<dyn AppEventHandler>,
    window_id: WindowId,
    event_loop: EventLoop<()>,
}

pub struct VulkanApp{
    pub device: Arc<Device>,
    pub queues: Vec<Arc<Queue>>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
}

pub trait VulkanAppBuilder{
    fn create_instance(&self) -> Arc<Instance>;
    fn build(
        &self,
        instance: Arc<Instance>,
        surface: Arc<Surface<Window>>,
    ) -> VulkanApp;
}

pub trait AppEventHandler {
    fn update(&self);
    fn render(&self);
}

pub trait AppEventHandlerFactory {
    fn create_event_handler(
        &self,
        device: Arc<Device>,
        queues: &Vec<Arc<Queue>>,
        swapchain_format: Format,
    ) -> Box<dyn AppEventHandler>;
}

pub enum UpdateFrequency {
    /// appropriate for GUI/static apps
    OnEvent,
    /// appropriate for game type apps
    Continuous,
}


impl App {

    pub fn new(
        update_frequency: UpdateFrequency,
        vulkan_app_builder: &dyn VulkanAppBuilder,
        event_handler_factory: &dyn AppEventHandlerFactory) -> App {

        // @TODO - add configuration options for logger
        logger::init();

        let event_loop = EventLoop::new();

        let vulkan_instance = vulkan_app_builder.create_instance();
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, vulkan_instance.clone()).unwrap();
        let window_id = surface.window().id();

        let vulkan_app = vulkan_app_builder.build(vulkan_instance, surface);

        let event_handler = event_handler_factory.create_event_handler(
            vulkan_app.device.clone(),
            &vulkan_app.queues,
            vulkan_app.swapchain.format(),
        );

        App {
            vulkan_app,
            event_handler,
            window_id,
            event_loop,
        }
    }

    pub fn create_frame_buffers(
        &self,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = self.vulkan_app.swapchain_images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        self.vulkan_app.swapchain_images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>()
    }

    /// Blocks until Application is complete
    pub fn run(self) {

        //let mut previous_frame_end = Some(sync::now(self.vulkan_app.device.clone()).boxed());

//        let draw = {
//            previous_frame_end
//                .as_mut()
//                .unwrap()
//                .cleanup_finished();
//        }
        //
//            if recreate_swapachain {
//                let dimensions: [u32: 2] = self.surface.window().inner_size().into();
//                let (new_swapchain, new_images) =
//                    match swapchain.recreate_with_dimensions(dimensions) {
//                        Ok(r) => r,
//                        Err(SwapchainCreationError:UnsupportedDiemnsions) => return,
//                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
//                    };
//
//                swap
//            }

//            let (image_num, suboptimal, aquire_future) =
//                match swapchain::aquire_next_image(swapchain.clone(), None) {
//                    Ok(r) => r,
//                    Err(AquireError::OutOfDate) => {
//                        recreate_swapchain = true;
//                        return;
//                    }
//                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
//                };
//
//            let suboptimal = false;
//            let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
//
//            let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
//                self.device.clone(),
//                self.queue.family(),
//            )
//            .unwrap();
//
//            builder.
//                begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
//                .unwrap()
//                .draw(
//                    pipeline.clone(),
//                    &dynamic_state,
//                    vertex_buffer.clone().
//                    (),
//                    (),
//                )
//                .unwrap()
//                .end_render_pass()
//                .unwrap();
//
//            let command_buffer = builder.build().unwrap();
//
//            let future = previous_frame_end
//                .take()
//                .unwrap()
//                .join(aquire_future)
//                .then_execute(self.queue.clone(), command_buffer)
//                .unwrap()
//                .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
//                .then_signal_fence_and_flush();
//
//            match future {
//                Ok(future) => {
//                    previous_frame_end = Some(future.boxed());
//                }
//                Err(FlushError::OutOfDate) => {
////                    recreate_swapchain = true;
//                    previous_frame_end = Some(sync::now(self.device.clone()).boxed());
//                }
//                Err(e) => {
//                    println!("Failed to flush future: {:?}", e);
//                    previous_frame_end = Some(sync::now(self.device.clone()).boxed());
//                }
//            }
//        };


        let id = self.window_id;
        let event_handler = self.event_handler;

        self.event_loop.run(move |event, _, control_flow| {

            event_handler.update();

            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id
                } if id == window_id => { *control_flow = ControlFlow::Exit; }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    window_id
                } if id == window_id => {
                    // recreate swapchain
                }
                Event::RedrawEventsCleared => {
                    event_handler.render();
                }
                _ => (),
            }
        });
    }
}


