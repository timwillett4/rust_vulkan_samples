#[macro_use] extern crate log;

mod logger;

pub fn run() {

    info!("Running App");

    logger::init_logger();
    os::create_window();
    vulkan::init_vulkan();

    debug!("This is a debug comment");
}

// @TODO come up with better name
mod os {

    use winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    pub fn create_window() {

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

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

mod vulkan {
    use vulkano::instance::Instance;
    use vulkano::instance::InstanceExtensions;
    use vulkano::instance::PhysicalDevice;
    use vulkano::device::Device;
    use vulkano::device::DeviceExtensions;
    use vulkano::device::Features;
    use vulkano::buffer::BufferUsage;
    use vulkano::buffer::CpuAccessibleBuffer;

    pub fn init_vulkan () {

        let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");

        let physical_device = PhysicalDevice::enumerate(&instance).next().expect("no device available");

        let queue_family = physical_device.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            Device::new(physical_device, &Features::none(), &DeviceExtensions::none(),
                        [(queue_family, 0.5)].iter().cloned()).expect("failed to create logical device")
        };

        let queue = queues.next().unwrap();

        let data = 12;
        let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data).expect("failed to create buffer");
    }

}
