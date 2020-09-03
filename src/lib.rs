#[macro_use] extern crate log;

pub extern crate vulkano;
pub extern crate vulkano_shaders;

pub mod logger;
pub mod vulkan_app;
pub mod vulkan_app_builder{
    pub mod single_graphics_queue;
}
