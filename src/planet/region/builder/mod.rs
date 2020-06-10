use super::{Planet, Region};
use crate::planet::{planet_idx, set_worldgen_status, REGION_HEIGHT, REGION_WIDTH};
use bracket_geometry::prelude::Point;
use bracket_random::prelude::RandomNumberGenerator;
mod heightmap;
mod primitive;
mod ramping;
mod strata;
mod water_features;
mod beaches;
mod buildings;
use legion::prelude::*;
pub use primitive::Primitive;
use crate::components::*;

pub fn builder(region: &mut Region, planet: &Planet, crash_site: Point) -> World {
    set_worldgen_status("Locating biome information");
    let biome_info = crate::raws::RAWS.read().biomes.areas[region.biome_raw_idx].clone();
    let biome = planet.biomes[region.biome_info_idx].clone();
    let mut pooled_water = vec![0u8; REGION_WIDTH as usize * REGION_HEIGHT as usize];
    let mut rng = RandomNumberGenerator::seeded(
        planet.perlin_seed + planet_idx(crash_site.x as usize, crash_site.y as usize) as u64,
    );

    set_worldgen_status("Establishing ground altitude");
    let mut hm = heightmap::build_empty_heightmap();
    heightmap::build_heightmap_from_noise(
        &mut hm,
        crash_site,
        planet.perlin_seed,
        planet.landblocks[planet_idx(crash_site.x as usize, crash_site.y as usize)].variance,
    );

    set_worldgen_status("Locating Sub-Biomes");
    heightmap::create_subregions(
        &mut rng,
        planet.landblocks[planet_idx(crash_site.x as usize, crash_site.y as usize)].variance,
        &mut hm,
        &mut pooled_water,
        &biome,
    );

    set_worldgen_status("Adding water features");
    water_features::just_add_water(planet, region, &mut pooled_water, &mut hm, &mut rng);
    water_features::set_water_tiles(region, &pooled_water, planet.water_height as usize);

    set_worldgen_status("Stratifying");
    let region_strata = strata::build_strata(&mut rng, &mut hm, &biome_info, planet.perlin_seed);

    set_worldgen_status("Layer cake is yummy");
    strata::layer_cake(&hm, region, &region_strata);

    set_worldgen_status("Ramping");
    ramping::build_ramps(region);

    set_worldgen_status("Beaches");
    beaches::build_beaches(region);

    set_worldgen_status("Building an ECS");
    let universe = Universe::new();
    let mut world = universe.create_world();
    world.insert(
        (Cordex {},),
        (0..1).map(|_| {
            (
                Position {
                    x: 128,
                    y: 128,
                    z: hm[(128 * REGION_WIDTH) + 128] as _,
                },
                CameraOptions {
                    zoom_level: 10,
                    mode: CameraMode::TopDown,
                },
            )
        }),
    );

    set_worldgen_status("Crashing the ship");
    let ship_loc = Point::new(128, 128);
    buildings::build_escape_pod(region, &ship_loc);

    set_worldgen_status("Trees");
    set_worldgen_status("Blight");
    set_worldgen_status("Trail of debris");
    set_worldgen_status("Escape pod");
    set_worldgen_status("Settlers");
    set_worldgen_status("Features");
    set_worldgen_status("Looking for the map");

    world
}
