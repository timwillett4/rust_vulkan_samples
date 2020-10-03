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
        memory::DeviceMemoryAllocError,
        pipeline::{
            GraphicsPipeline,
            GraphicsPipelineAbstract,
        },
        sync,
        sync::{FlushError, GpuFuture},
        swapchain::{SwapchainCreationError, AcquireError},
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

        let graphics_queue = vulkan_app.queues.get(0).ok_or("Device has no available queues")?.clone();

        let surface_format = {
            let caps = surface.capabilities(device.physical_device())?;

            caps.supported_formats[0].0
        };

        let render_pass = SimpleTriangleEventHandlerFactory::create_renderpass(&device, surface_format)?;

        let graphics_pipeline = SimpleTriangleEventHandlerFactory::create_pipeline(&device, render_pass.clone())?;

        let vertex_buffer = SimpleTriangleEventHandlerFactory::create_vertex_buffer(&device)?;

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
            recreate_render_state: false,
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
    ) -> Result<Arc<dyn GraphicsPipelineAbstract + Send + Sync>, Box<dyn Error>> {

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

        let vs = vs::Shader::load(device.clone())?;
        let fs = fs::Shader::load(device.clone())?;

        let subpass = Subpass::from(render_pass, 0).ok_or("Unable to build subpass")?;

        Ok(Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(subpass)
            .build(device.clone())?))
    }

    fn create_vertex_buffer(device: &Arc<Device>) -> Result<Arc<dyn BufferAccess + Send + Sync>, DeviceMemoryAllocError> {

        vulkano::impl_vertex!(Vertex, position);

        Ok(Arc::new(CpuAccessibleBuffer::from_iter(
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
        )?))
    }
}

struct SimpleTriangleEventHandler{
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    render_state: RenderState,
    recreate_render_state: bool,
}

impl AppEventHandler for SimpleTriangleEventHandler {

    fn on_update(&mut self) {}

    fn on_window_resize(&mut self, _width: u32, _height: u32) -> Result<(), Box<dyn Error>> {
        self.recreate_render_state = true;
        Ok(())
    }

    fn on_redraw(&mut self) -> Result<(), Box<dyn Error>> {

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_render_state {
            match self.render_state.recreate() {
                Ok(()) => self.recreate_render_state = false,
                Err(e) => match e.downcast_ref::<SwapchainCreationError>() {
                    // Return okay to indicate non-fatal error
                    Some(SwapchainCreationError::UnsupportedDimensions) => return Ok(()),
                    _ => return Err(e),
                },
            };
        };

        let next_image_result = match self.render_state.acquire_next_image() {
            Ok(result) => result,
            Err(AcquireError::OutOfDate) => {
                self.recreate_render_state = true;
                return Ok(());
            },
            Err(e) => return Err(Box::new(e)),
        };

        self.recreate_render_state = next_image_result.suboptimal;

        let command_buffer = self.create_command_buffer(
            next_image_result.framebuffer,
            &self.render_state.dynamic_state,
        )?;

        let future = self.previous_frame_end
            .take()
            .expect("Previous frame end should never be none")
            .join(next_image_result.acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(self.graphics_queue.clone(), self.render_state.swapchain.clone(), next_image_result.image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
                Ok(())
            }
            Err(FlushError::OutOfDate) => {

                warn!("Present future error: 'FlushError::OutOfDate'");
                self.recreate_render_state = true;
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
    ) -> Result<impl CommandBuffer<PoolAlloc = StandardCommandPoolAlloc>, Box<dyn Error>> {

        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.graphics_queue.family(),
        )?;

        builder.
            begin_render_pass(framebuffer, false, clear_values)?
            .draw(
                self.graphics_pipeline.clone(),
                &dynamic_state,
                vec![self.vertex_buffer.clone()],
                (),
                (),
            )?
            .end_render_pass()?;

        Ok(builder.build()?)
    }
}
