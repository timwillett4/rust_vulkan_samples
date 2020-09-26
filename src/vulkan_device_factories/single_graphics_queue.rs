use crate::vulkan_app::DeviceFactory;

use std::{
    sync::Arc,
    error::Error,
};

use winit::window::Window;

use vulkano::{
    device::{
        Device,
        Features,
        Queue,
    },
    instance::{
        Instance,
        PhysicalDevice,
        QueueFamily
    },
    swapchain::Surface,
};

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
    ) -> Result<(Arc<Device>, Vec<Arc<Queue>>), Box<dyn Error>> {

        let (physical_device, compatible_graphics_queue_family) = {
            PhysicalDevice::enumerate(&instance).find_map(
                |physical_device| -> Option<(PhysicalDevice, QueueFamily)> {

                    let compatible_graphics_queue_family = physical_device.queue_families().find(
                        |queue_family| -> bool {
                            queue_family.supports_graphics()
                            && surface.is_supported(*queue_family).unwrap_or(false)
                    });

                    match compatible_graphics_queue_family {
                        Some(queue_family) => Some((physical_device, queue_family)),
                        None => None,
                    }
            }).ok_or("Unable to find compatible device")
        }?;

        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, queues) = Device::new(
            physical_device,
            &Features::none(),
            &device_extensions,
            [(compatible_graphics_queue_family, 0.5)].iter().cloned()
        )?;

        let queues = queues.collect::<Vec<_>>();

        Ok((device, queues))
    }
}
