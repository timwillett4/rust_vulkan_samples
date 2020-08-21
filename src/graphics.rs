use std::sync::Arc;

use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::instance::QueueFamily;
//use vulkano::device::QueuesIter;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::Queue;
//use vulkano::buffer::BufferUsage;
//use vulkano::buffer::CpuAccessibleBuffer;
//use vulkano::command_buffer::AutoCommandBufferBuilder;
//use vulkano::command_buffer::CommandBuffer;
//use vulkano::sync::GpuFuture;


#[derive(Clone)]
struct VulkanDevice {
    /// Takes ownership of instance and physical device
    logical_device : Arc<Device>,
    queue : Arc<Queue>
    // should be iterator???
    //queues : QueuesIter,
}

impl VulkanDevice {

    pub fn new() -> VulkanDevice {

        debug!("Initing Vulkan");

        let instance = Instance::new(None, &InstanceExtensions::none(), None)
            .expect("failed to create instance");

        let (physical_device, queue_family) : (PhysicalDevice, QueueFamily) = PhysicalDevice::enumerate(&instance)
            .find_map(|physical_device| -> Option<(PhysicalDevice, QueueFamily)> {
                match physical_device.queue_families().find(|queue_family| -> bool { queue_family.supports_graphics() }) {
                    Some(queue_family) => Some((physical_device, queue_family)),
                    None => None,
                }}).expect("couldn't find a graphical queue family");


        let (logical_device, mut queues) = {
            Device::new(physical_device, &Features::none(), &DeviceExtensions::none(),
                        [(queue_family, 0.5)].iter().cloned()).expect("failed to create logical device")
        };

        let queue = queues.next().expect("couldn't fin a graphics queues");

        VulkanDevice {
            logical_device,
            queue,
        }
    }
/*
    pub fn run (&self) {

        let source_content = 0 .. 64;
        let source = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false,
                                                    source_content).expect("failed to create buffer");

        let dest_content = (0 .. 64).map(|_| 0);
        let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false,
                                                dest_content).expect("failed to create buffer");

        let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
        builder.copy_buffer(source.clone(), dest.clone()).unwrap();
        let command_buffer = builder.build().unwrap();

        let finished = command_buffer.execute(queue.clone()).unwrap();

        finished.then_signal_fence_and_flush().unwrap().
            wait(None).unwrap();

        let src_content = source.read().unwrap();
        let dest_content = dest.read().unwrap();
        assert_eq!(&*src_content, &*dest_content);

        debug!("Success!");
    }
*/
}
