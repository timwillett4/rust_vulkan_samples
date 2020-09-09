#[macro_use] extern crate log;

use vulkan_samples::vulkan_app::{
    AppEventHandler,
    AppEventHandlerFactory,
    UpdateFrequency,
    VulkanApp,
    SwapchainFactory,
};

use vulkan_samples::vulkan_app_builder::single_graphics_queue::{
    DefaultInstanceFactory,
    DefaultSwapchainFactory,
    SingleGraphicsQueueDeviceFactory,
};

use std::sync::Arc;

use winit::window::Window;

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
        BufferAccess
    },
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBuffer,
        DynamicState
    },
    command_buffer::pool::standard::StandardCommandPoolAlloc,
    device::{
        Device,
        Queue
    },
    format::Format,
    framebuffer::{
        Subpass,
        FramebufferAbstract,
        RenderPassAbstract,
    },
    image::SwapchainImage,
    pipeline::{
        GraphicsPipeline,
        GraphicsPipelineAbstract,
    },
    swapchain::{
        SwapchainAcquireFuture,
        Surface,
        Swapchain,
    },
    sync,
    sync::{FlushError, GpuFuture},
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
fn main() {

    let app = VulkanApp::new(
        UpdateFrequency::Continuous,
        DefaultInstanceFactory::new(),
        SingleGraphicsQueueDeviceFactory::new(),
        SimpleTriangleEventHandlerFactory::new(),
    );

    app.run()
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

struct SimpleTriangleEventHandlerFactory{}

struct SimpleTriangleEventHandler{
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    graphics_queue: Arc<Queue>,
    vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    swapchain_factory: Box<dyn SwapchainFactory>,
}

impl AppEventHandlerFactory for SimpleTriangleEventHandlerFactory {

    fn create_event_handler(
        &self,
        device: Arc<Device>,
        queues: &Vec<Arc<Queue>>,
        surface: Arc<Surface<Window>>,
    ) -> Box<dyn AppEventHandler> {

        let surface_format = {
            let caps = surface.capabilities(device.physical_device()).expect("failsd to get surface capabilties");

            caps.supported_formats[0].0
        };

        let render_pass = SimpleTriangleEventHandlerFactory::create_renderpass(&device, surface_format);

        let graphics_pipeline = SimpleTriangleEventHandlerFactory::create_pipeline(&device, render_pass.clone());

        let vertex_buffer = SimpleTriangleEventHandlerFactory::create_vertex_buffer(&device);

        let graphics_queue = queues[0].clone();

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Box::new(SimpleTriangleEventHandler{
            device,
            surface,
            graphics_queue,
            vertex_buffer,
            render_pass,
            graphics_pipeline,
            previous_frame_end,
            swapchain_factory: DefaultSwapchainFactory::new(),
        })
    }
}

impl SimpleTriangleEventHandlerFactory {

    fn new() -> Box<dyn AppEventHandlerFactory> {

        Box::new(SimpleTriangleEventHandlerFactory{})
    }

    fn create_renderpass(
        device: &Arc<Device>,
        format: Format,
    ) -> Arc<dyn RenderPassAbstract + Send + Sync> {

        Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {

                    color: {
                        load: Clear,
                        store: Store,
                        format: format,
                        samples: 1,
                    }
                },

                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap()
        )
    }

    fn create_pipeline(
        device: &Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>
    ) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {

        mod vs {
            vulkano_shaders::shader!{
                ty: "vertex",
                src: "
                    #version 450

                    layout(location = 0) in vec2 position;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                "
            }
        }

        mod fs {
            vulkano_shaders::shader!{
                ty: "fragment",
                src: "
                    #version 450

                    layout(location = 0) out vec4 f_color;

                    void main() {
                        f_color = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                "
            }
        }

        let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .build(device.clone())
                .unwrap()
        )
    }

    fn create_vertex_buffer(device: &Arc<Device>) -> Arc<dyn BufferAccess + Send + Sync> {

        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex { position: [-0.5, -0.25] },
                Vertex { position: [ 0.0,  0.5] },
                Vertex { position: [ 0.25, -0.1] },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    }
}

impl AppEventHandler for SimpleTriangleEventHandler {

    fn create_swapchain(&self) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        self.swapchain_factory.create_swapchain(
            self.device.clone(),
            &self.graphics_queue.clone(),
            self.surface.clone(),
        )
    }

    fn create_renderpass(&self) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        self.render_pass.clone()
    }

    fn create_dynamic_state(&self) -> DynamicState {
        DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        }
    }

    fn update(&self) {

    }

    fn render(
        &mut self,
        swapchain: Arc<Swapchain<Window>>,
        acquire_future: SwapchainAcquireFuture<Window>,
        image_num: usize,
        framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
        dynamic_state: &DynamicState,
    ) {

        let command_buffer = self.create_command_buffer(
            framebuffer,
            dynamic_state,
        );

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.graphics_queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                //render_state.recreate();
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }
}

impl SimpleTriangleEventHandler {

    fn create_command_buffer(
        &self,
        framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
        dynamic_state: &DynamicState,
    ) -> impl CommandBuffer<PoolAlloc = StandardCommandPoolAlloc> {

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.graphics_queue.family(),
        )
        .unwrap();

        builder.
            begin_render_pass(framebuffer, false, clear_values)
            .unwrap()
            .draw(
                self.graphics_pipeline.clone(),
                &dynamic_state,
                vec![self.vertex_buffer.clone()],
                (),
                (),
            )
            .unwrap()
            .end_render_pass()
            .unwrap();

        builder.build().unwrap()
    }
}
