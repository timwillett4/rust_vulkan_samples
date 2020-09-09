
use crate::logger;

use std::sync::Arc;

use winit::{
    event::{
        Event,
        WindowEvent,
    },
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::{
        WindowBuilder,
        Window,
    },
};

use vulkano_win::VkSurfaceBuild;

use vulkano::{
    command_buffer::DynamicState,
    device::{
        Device,
        Queue
    },
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPassAbstract
    },
    image::SwapchainImage,
    instance::Instance,
    pipeline::viewport::Viewport,
    swapchain,
    swapchain::{
        AcquireError,
        SwapchainAcquireFuture,
        Surface,
        Swapchain,
    },
};


pub struct VulkanApp{
    device: Arc<Device>,
    queues: Vec<Arc<Queue>>,
    surface: Arc<Surface<Window>>,
    event_loop: EventLoop<()>,
    update_frequency: UpdateFrequency,
    event_handler_factory: Box<dyn AppEventHandlerFactory>,
}

pub trait InstanceFactory {
    fn create_instance(&self) -> Arc<Instance>;
}

pub trait DeviceFactory{
    fn create_device(
        &self,
        instance: Arc<Instance>,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Device>, Vec<Arc<Queue>>);
}

pub trait SwapchainFactory {
    fn create_swapchain(
        &self,
        device: Arc<Device>,
        queue: &Arc<Queue>,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>);
}

pub trait AppEventHandler {
    fn create_swapchain(&self) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>);
    fn create_renderpass(&self) -> Arc<dyn RenderPassAbstract + Send + Sync>;
    fn create_dynamic_state(&self) -> DynamicState;
    fn update(&self);
    fn render(
        &mut self,
        swapchain: Arc<Swapchain<Window>>,
        acquire_future: SwapchainAcquireFuture<Window>,
        image_num: usize,
        framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
        dynamic_state: &DynamicState,
    );
}

pub trait AppEventHandlerFactory {
    fn create_event_handler(
        &self,
        device: Arc<Device>,
        queues: &Vec<Arc<Queue>>,
        surface: Arc<Surface<Window>>,
    ) -> Box<dyn AppEventHandler>;
}

pub enum UpdateFrequency {
    /// appropriate for GUI/static apps
    OnEvent,
    /// appropriate for game type apps
    Continuous,
}

struct RenderState {
    swapchain: Arc<Swapchain<Window>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    surface: Arc<Surface<Window>>,
    dynamic_state: DynamicState,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl RenderState {
    pub fn new(
        event_handler: &Box<dyn AppEventHandler>,
        surface: Arc<Surface<Window>>,
    ) -> RenderState {

        let (swapchain, swapchain_images) = event_handler.create_swapchain();

        let mut dynamic_state = event_handler.create_dynamic_state();

        let render_pass = event_handler.create_renderpass();

        let framebuffers = VulkanApp::create_frame_buffers(
            render_pass.clone(),
            &mut dynamic_state,
            &swapchain_images,
        );

        RenderState {
            swapchain,
            framebuffers,
            surface,
            dynamic_state,
            render_pass,
        }
    }

    pub fn recreate(
        &mut self,
    ) {
        let (swapchain, swapchain_images) = RenderState::recreate_swapchain(self.swapchain.clone(), self.surface.clone());

        self.swapchain = swapchain;

        self.framebuffers = VulkanApp::create_frame_buffers(
            self.render_pass.clone(),
            &mut self.dynamic_state,
            &swapchain_images,
        );
    }

    fn recreate_swapchain(
        swapchain: Arc<Swapchain<Window>>,
        surface: Arc<Surface<Window>>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {

        let dimensions: [u32; 2] = surface.window().inner_size().into();

        swapchain.recreate_with_dimensions(dimensions).unwrap()
    }
}

impl VulkanApp {

    pub fn new(
        update_frequency: UpdateFrequency,
        instance_factory: Box<dyn InstanceFactory>,
        device_factory: Box<dyn DeviceFactory>,
        event_handler_factory: Box<dyn AppEventHandlerFactory>) -> VulkanApp {
        // @TODO - add configuration options for logger
        logger::init();

        let instance = instance_factory.create_instance();

        let event_loop = EventLoop::new();

        let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

        let (device, queues) = device_factory
            .create_device(instance.clone(), surface.clone());

        VulkanApp {
            device,
            queues,
            surface,
            event_loop,
            update_frequency,
            event_handler_factory,
        }
    }

    // @TODO this is to coupled betweeen vulkan stuff and
    // winit stuff
    // use events (ie init, on resize, on render...)
    /// Blocks until Application is complete
    pub fn run(self) {

        let mut event_handler = self.event_handler_factory.create_event_handler(
            self.device.clone(),
            &self.queues,
            self.surface.clone(),
        );

        let id = self.surface.window().id();

        let mut render_state = RenderState::new(&event_handler, self.surface);

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
                    render_state.recreate();
                }
                Event::RedrawEventsCleared => {
                    VulkanApp::render(
                        &mut render_state,
                        &mut event_handler
                    );
                }
                _ => (),
            }
        });
    }


    fn render(
        render_state: &mut RenderState,
        event_handler: &mut Box<dyn AppEventHandler>,
    ) {
        // @TODO - rather then being in app create some type of swapchain abstraction
        let (image_num, suboptimal, acquire_future) =

            match swapchain::acquire_next_image(render_state.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    render_state.recreate();
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            render_state.recreate();
        }

        let framebuffer = render_state.framebuffers[image_num].clone();

        // @TODO - this code below is not generic enough
        // extract/assign responsibility better after
        // doing a couple more examples and getting better understanding
        // of what is common and what is distinct
        event_handler.render(
            render_state.swapchain.clone(),
            acquire_future,
            image_num,
            framebuffer,
            &render_state.dynamic_state,
        );
    }

    fn create_frame_buffers(
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
        swapchain_images: &Vec<Arc<SwapchainImage<Window>>>,
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {

        let dimensions = swapchain_images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };

        dynamic_state.viewports = Some(vec![viewport]);

        swapchain_images
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
}
