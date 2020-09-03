use crate::vulkan_app::{VulkanApp, VulkanAppBuilder};

use std::sync::Arc;

use vulkano::instance::{Instance, PhysicalDevice, QueueFamily};
use vulkano::swapchain::{
    ColorSpace,
    FullscreenExclusive,
    PresentMode,
    Surface,
    SurfaceTransform,
    Swapchain,
};
use vulkano::image::ImageUsage;
use vulkano::device::{Device, Features};
use winit::window::Window;

pub struct SingleGraphicsQueueAppBuilder{}

impl SingleGraphicsQueueAppBuilder {

    pub fn new() -> SingleGraphicsQueueAppBuilder {
        SingleGraphicsQueueAppBuilder{}
    }
}

impl VulkanAppBuilder for SingleGraphicsQueueAppBuilder {

    fn create_instance(&self) -> Arc<Instance> {

        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create Vulkan Instance")
    }

    fn build(
        &self,
        instance: Arc<Instance>,
        surface: Arc<Surface<Window>>,
    ) -> VulkanApp {

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

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &device_extensions,
            [(queue_family, 0.5)].iter().cloned()
        )
        .expect("failed to create logical device");

        let (swapchain, swapchain_images) = {
            let caps = surface.capabilities(physical_device).expect("failsd to get surface capabilties");

            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions = surface.window().inner_size().into();

            let queue = queues.next().expect("couldn't find a graphics queues");

            Swapchain::new(
                device.clone(),
                surface.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                ImageUsage::color_attachment(),
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            )
            .expect("failed to create swapchain")
        };

        let queues = queues.collect::<Vec<_>>();

        VulkanApp {
            device,
            queues,
            swapchain,
            swapchain_images,
        }
    }
}
