mod loader;
use wgpu::{Device, ShaderModule};

pub struct Shaders {
    shaders: Vec<ShaderModule>
}

impl Shaders {
    pub(crate) fn new() -> Self {
        Self {
            shaders: Vec::new()
        }
    }

    pub fn register_include(&mut self, source: wgpu::ShaderModuleSource) -> usize {
        let sm = loader::from_spv(source);
        let idx = self.shaders.len();
        self.shaders.push(sm);
        idx
    }

    pub fn get_module(&self, id: usize) -> &wgpu::ShaderModule {
        println!("Access shader {}", id);
        &self.shaders[id]
    }
}