use crate::modes::{
    playgame::GrassPass, playgame::ModelsPass, playgame::TerrainPass, GBuffer, Palette,
};
use parking_lot::RwLock;
use std::thread;

lazy_static! {
    pub static ref LOADER: RwLock<LoaderState> = RwLock::new(LoaderState::new());
}

pub struct LoaderState {
    pub progress: f32,
    pub status: String,
    pub done: bool,
    pub palette: Option<Palette>,
    pub g_buffer: Option<GBuffer>,
    pub terrain_pass: Option<TerrainPass>,
    pub model_pass: Option<ModelsPass>,
    pub grass_pass: Option<GrassPass>,
}

impl LoaderState {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            status: "Randomly Flipping Bits...".to_string(),
            done: false,
            g_buffer: None,
            palette: None,
            terrain_pass: None,
            model_pass: None,
            grass_pass: None,
        }
    }

    pub fn start_loading() {
        thread::spawn(|| {
            LOADER.write().update(0.01, "Loading Raw Files", false);

            crate::load_raws();

            LOADER.write().update(0.02, "Fingerpainting", false);
            let palette = super::super::Palette::new();

            LOADER.write().update(0.03, "Baking Materials", false);
            {
                let mut rawlock = crate::RAWS.write();
                let mats = rawlock.materials.materials.clone();
                rawlock.matmap.build(&mats, &palette);
            }

            let gbuffer = GBuffer::new();
            LOADER.write().g_buffer = Some(gbuffer);

            LOADER.write().update(0.04, "Terrain Renderer", false);
            let terrain_pass = TerrainPass::new(&palette);

            LOADER.write().update(0.05, "Wavefront Models", false);
            let models = crate::modes::playgame::Models::load_models(&palette);
            let model_pass = ModelsPass::new(&palette, models, &terrain_pass.uniforms);

            LOADER.write().update(0.1, "Mowing the lawn", false);
            let grassy_gnoll = crate::modes::playgame::GrassPass::new(&terrain_pass.uniforms);

            LOADER.write().terrain_pass = Some(terrain_pass);
            LOADER.write().model_pass = Some(model_pass);
            LOADER.write().grass_pass = Some(grassy_gnoll);

            // Finish up by moving the palette
            LOADER.write().palette = Some(palette);
            LOADER.write().update(1.00, "Built all the things", true);
        });
    }

    fn update(&mut self, progress: f32, status: &str, is_done: bool) {
        self.progress = progress;
        self.status = status.to_string();
        self.done = is_done;
    }
}
