use std::sync::Arc;

use winit::window::Window;

use vulkano::{
    command_buffer::DynamicState,
    device::{
        Device,
        Queue,
    },
    framebuffer::{
        Framebuffer,
        FramebufferAbstract,
        RenderPassAbstract,
    },
    image::{
        ImageUsage,
        SwapchainImage
    },
    instance::Instance,
    pipeline::viewport::Viewport,
    swapchain,
    swapchain::{
        ColorSpace,
        FullscreenExclusive,
        PresentMode,
        Surface,
        SurfaceTransform,
        Swapchain,
        SwapchainAcquireFuture,
    },
};

pub struct VulkanApp {
    pub device: Arc<Device>,
    pub queues: Vec<Arc<Queue>>,
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

// @TODO consider seperating device/queues and swapchain
// since non-windowed vulkan apps are valid and may
// be better for testing

impl VulkanApp {
    pub fn new(
        instance_factory: Box<dyn InstanceFactory>,
        device_factory: Box<dyn DeviceFactory>,
        window: Window,
    ) -> (VulkanApp, Arc<Surface<Window>>) {
        let instance = instance_factory.create_instance();
        let surface = vulkano_win::create_vk_surface(window, instance.clone()).unwrap();
        let (device, queues) = device_factory.create_device(instance, surface.clone());

        (VulkanApp {device, queues}, surface)
    }
}

pub struct DefaultInstanceFactory{}

impl DefaultInstanceFactory { pub fn new() -> Box<dyn InstanceFactory> {
        Box::new(DefaultInstanceFactory{})
    }
}

impl InstanceFactory for DefaultInstanceFactory {
    // @TODO should take app info, extensions, layers
    fn create_instance(&self) -> Arc<Instance> {

        let extensions = vulkano_win::required_extensions();

        Instance::new(None, &extensions, None).expect("failed to create Vulkan Instance")
    }
}

pub struct DefaultSwapchainFactory{}

impl DefaultSwapchainFactory { pub fn new() -> Box<dyn SwapchainFactory> {
        Box::new(DefaultSwapchainFactory{})
    }
}

impl SwapchainFactory for DefaultSwapchainFactory {
    fn create_swapchain(
        &self,
        device: Arc<Device>,
        queue: &Arc<Queue>,
        surface: Arc<Surface<Window>>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {

        let caps = surface.capabilities(device.physical_device()).expect("failsd to get surface capabilties");

        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions = surface.window().inner_size().into();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .expect("failed to create swapchain")
    }
}

pub struct RenderState {
    pub swapchain: Arc<Swapchain<Window>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    surface: Arc<Surface<Window>>,
    pub dynamic_state: DynamicState,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl RenderState {

    pub fn new(
        device: Arc<Device>,
        swapchain_factory: Box<dyn SwapchainFactory>,
        swapchain_queue: &Arc<Queue>,
        surface: Arc<Surface<Window>>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    ) -> RenderState {

        let (swapchain, swapchain_images) = swapchain_factory.create_swapchain(device, &swapchain_queue, surface.clone());

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let framebuffers = RenderState::create_frame_buffers(
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

        self.framebuffers = RenderState::create_frame_buffers(
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

    pub fn acquire_next_image(&mut self)
        -> (usize, SwapchainAcquireFuture<Window>, Arc<dyn FramebufferAbstract + Send + Sync>) {

        let (image_num, suboptimal, acquire_future) =

            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                //                @TODO - this error needs to be passed on to event handler
//                Err(AcquireError::OutOfDate) => {
//                    self.recreate();
//                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate();
        }

        let frame_buffer = self.framebuffers[image_num].clone();

        (image_num, acquire_future, frame_buffer)
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
