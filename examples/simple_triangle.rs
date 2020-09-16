#[macro_use] extern crate log;

use vulkan_samples::{
        app::{
            App,
            AppEventHandlerFactory,
            AppEventHandler,
            UpdateFrequency,
        },
        vulkan_app::{
            DefaultInstanceFactory,
            DefaultSwapchainFactory,
            RenderState,
            VulkanApp,
        },
        vulkan_device_factories::single_graphics_queue::SingleGraphicsQueueDeviceFactory,
};

use std::{
        sync::Arc,
        error::Error,
};

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
            RenderPassCreationError,
        },
        pipeline::{
            GraphicsPipeline,
            GraphicsPipelineAbstract,
        },
        sync,
        sync::{FlushError, GpuFuture},
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
fn main() {
    match  App::new(
        UpdateFrequency::Continuous,
        SimpleTriangleEventHandlerFactory::new(),
    ) {
        Ok(app) => app.run(),
        Err(e) => error!("{}", e),
    }
}

struct SimpleTriangleEventHandlerFactory {}

impl AppEventHandlerFactory for SimpleTriangleEventHandlerFactory {

    fn create_event_handler(&self, window: Window) -> Result<Box<dyn AppEventHandler>, Box<dyn Error>> {
        let (vulkan_app, surface) = VulkanApp::new(
            DefaultInstanceFactory::new(),
            SingleGraphicsQueueDeviceFactory::new(),
            window,
        )?;

        let device = vulkan_app.device;
        // @TODO - this should actually check for first queue that supports graphics
        let graphics_queue = vulkan_app.queues[0].clone();

        let surface_format = {
            let caps = surface.capabilities(device.physical_device()).expect("failed to get surface capabilties");

            caps.supported_formats[0].0
        };

        let render_pass = SimpleTriangleEventHandlerFactory::create_renderpass(&device, surface_format)?;

        let graphics_pipeline = SimpleTriangleEventHandlerFactory::create_pipeline(&device, render_pass.clone());

        let vertex_buffer = SimpleTriangleEventHandlerFactory::create_vertex_buffer(&device);

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // @TODO - this perhaps needs to be some type of generalized interface?
        let render_state = RenderState::new(
            device.clone(),
            DefaultSwapchainFactory::new(),
            &graphics_queue,
            surface,
            render_pass,
        )?;

        Ok(Box::new(SimpleTriangleEventHandler{
            device,
            graphics_queue,
            graphics_pipeline,
            vertex_buffer,
            previous_frame_end,
            render_state,
        }))
    }
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl SimpleTriangleEventHandlerFactory {

    fn new() -> Box<dyn AppEventHandlerFactory> {

        Box::new(SimpleTriangleEventHandlerFactory{})
    }

    fn create_renderpass(
        device: &Arc<Device>,
        format: Format,
    ) -> Result<Arc<dyn RenderPassAbstract + Send + Sync>, RenderPassCreationError> {

        let render_pass = vulkano::single_pass_renderpass!(
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
        )?;

        Ok(Arc::new(render_pass))
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

struct SimpleTriangleEventHandler{
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    render_state: RenderState,
}

impl AppEventHandler for SimpleTriangleEventHandler {

    fn on_update(&mut self) {}

    fn on_window_resize(&mut self, _width: u32, _height: u32) -> Result<(), Box<dyn Error>> {
       self.render_state.recreate()
    }

    fn on_redraw(&mut self) -> Result<(), Box<dyn Error>> {

        let (image_num, acquire_future, frame_buffer) = self.render_state.acquire_next_image()?;

        let command_buffer = self.create_command_buffer(
            frame_buffer,
            &self.render_state.dynamic_state,
        );

        let future = self.previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(self.graphics_queue.clone(), self.render_state.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
                Ok(())
            }
            Err(FlushError::OutOfDate) => {
                // @TODO - this maybe neds to be handled better
                self.render_state.recreate()?;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());

                Ok(())
            }
            Err(e) => {
                warn!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());

                Ok(())
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
