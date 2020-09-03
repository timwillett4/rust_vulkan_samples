#[macro_use] extern crate log;

use vulkan_samples::vulkan_app::{App, AppEventHandler, AppEventHandlerFactory, UpdateFrequency};
use vulkan_samples::vulkan_app_builder::single_graphics_queue::SingleGraphicsQueueAppBuilder;
use std::sync::Arc;

use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::framebuffer::{Subpass, FramebufferAbstract, RenderPassAbstract};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, BufferAccess};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
fn main() {

    let app = App::new(
        UpdateFrequency::Continuous,
        &SingleGraphicsQueueAppBuilder::new(),
        &SimpleTriangleEventHandlerFactory::new(),
    );


    // @TODO - this should be in app common code

    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
        compare_mask: None,
        write_mask: None,
        reference: None,
    };

    //let mut framebuffers = app.create_frame_buffers(render_pass, &mut dynamic_state);
    // this should be render function

   /* let command_buffer = create_command_buffer(
                            device,
                            graphics_queue,
                            graphics_pipeline,
                            framebuffer,
                            &mut dynamic_state,
                        ); */
    app.run()
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

struct SimpleTriangleEventHandlerFactory{}

struct SimpleTriangleEventHandler{
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    vertex_buffer: Arc<dyn BufferAccess + Send + Sync>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl AppEventHandlerFactory for SimpleTriangleEventHandlerFactory {

    fn create_event_handler(
        &self,
        device: Arc<Device>,
        queues: &Vec<Arc<Queue>>,
        swapchain_format: Format,
    ) -> Box<dyn AppEventHandler> {

        let render_pass = SimpleTriangleEventHandlerFactory::create_renderpass(&device, swapchain_format);
        let graphics_pipeline = SimpleTriangleEventHandlerFactory::create_pipeline(&device, render_pass.clone());
        let vertex_buffer = SimpleTriangleEventHandlerFactory::create_vertex_buffer(&device);

        let graphics_queue = queues[0].clone();

        Box::new(SimpleTriangleEventHandler{
            device,
            graphics_queue,
            vertex_buffer,
            render_pass,
            graphics_pipeline,
        })
    }
}

impl SimpleTriangleEventHandlerFactory {

    fn new() -> SimpleTriangleEventHandlerFactory {

        SimpleTriangleEventHandlerFactory{}
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

    fn update(&self) {

    }

    fn render(&self) {

    }
}

impl SimpleTriangleEventHandler {

    fn create_command_buffer(
        &self,
        framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
        dynamic_state: &mut DynamicState,
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
