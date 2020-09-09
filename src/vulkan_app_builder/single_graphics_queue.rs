use crate::vulkan_app::{
    DeviceFactory,
    InstanceFactory,
    SwapchainFactory
};

use std::sync::Arc;

use winit::window::Window;

use vulkano::{
    device::{
        Device,
        Features,
        Queue
    },
    image::{
        ImageUsage,
        SwapchainImage
    },
    instance::{
        Instance,
        PhysicalDevice,
        QueueFamily
    },
    swapchain::{
        ColorSpace,
        FullscreenExclusive,
        PresentMode,
        Surface,
        SurfaceTransform,
        Swapchain,
    },
};

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

pub struct SingleGraphicsQueueDeviceFactory{}

impl SingleGraphicsQueueDeviceFactory {

    pub fn new() -> Box<dyn DeviceFactory> {
        Box::new(SingleGraphicsQueueDeviceFactory{})
    }
}

impl DeviceFactory for SingleGraphicsQueueDeviceFactory {

    fn create_device(
        &self,
        instance: Arc<Instance>,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Device>, Vec<Arc<Queue>>) {

        let (physical_device, queue_family) = {
            PhysicalDevice::enumerate(&instance).find_map(
                |physical_device| -> Option<(PhysicalDevice, QueueFamily)> {

                    let queue_family = physical_device.queue_families().find(
                        |queue_family| -> bool {
                            queue_family.supports_graphics()
                            && surface.is_supported(*queue_family).unwrap_or(false)
                    });

                    match queue_family {
                        Some(queue_family) => Some((physical_device, queue_family)),
                        None => None,
                    }
            })
            .expect("couldn't find a graphical queue family")
        };

        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, queues) = Device::new(
            physical_device,
            &Features::none(),
            &device_extensions,
            [(queue_family, 0.5)].iter().cloned()
        )
        .expect("failed to create logical device");

        let queues = queues.collect::<Vec<_>>();

        (device, queues)
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
