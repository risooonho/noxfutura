use crate::planet::{REGION_TILES_COUNT, REGION_HEIGHT, REGION_WIDTH};
use bracket_geometry::prelude::*;
use bracket_noise::prelude::*;
use bracket_random::prelude::*;

pub fn build_empty_heightmap() -> Vec<u8> {
    vec![0u8 ; (REGION_WIDTH * REGION_HEIGHT) as usize]
}

fn sphere_vertex(altitude: f32, lat: f32, lon: f32) -> (f32, f32, f32) {
    let rlat = lat * 0.0174533;
    let rlon = lon * 0.0174533;
    (
        altitude * f32::cos(rlat) * f32::cos(rlon),
        altitude * f32::cos(rlat) * f32::sin(rlon),
        altitude * f32::sin(rlat),
    )
}

pub fn build_heightmap_from_noise(hm : &mut Vec<u8>, crash_site: Point, perlin_seed : u64) {
    use crate::planet::noise_helper::*;

    let mut noise = FastNoise::seeded(perlin_seed);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(10);
    noise.set_fractal_gain(0.4);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(0.01);

    for y in 0..REGION_HEIGHT {
        let lat = noise_lat(crash_site.y, y);
        for x in 0..REGION_WIDTH {
            let lon = noise_lon(crash_site.x, x);
            let sphere_coords = sphere_vertex(1000.0, lat, lon);
            let nh = noise.get_noise3d(sphere_coords.0, sphere_coords.1, sphere_coords.2);
            let altitude = noise_to_planet_height(nh);
            let cell_idx = ((y * REGION_WIDTH) + x) as usize;
            hm[cell_idx] = altitude;
        }
    }
}

pub fn create_subregions(
    rng: &mut RandomNumberGenerator,
    variance : u8,
    hm : &mut Vec<u8>,
    water: &mut Vec<u8>,
    biome: &crate::planet::Biome
) {
    let center_point = Point::new(REGION_WIDTH/2, REGION_HEIGHT/2);
    let n_subregions = 64 + rng.roll_dice(1, 20) + (variance as i32 * 4);

    // Set each heightmap tile to be a member of a sub-region
    let mut centroids : Vec<Point> = Vec::new();
    for _ in 0..n_subregions {
        centroids.push(
            Point::new(
                rng.roll_dice(1, REGION_WIDTH)-1,
                rng.roll_dice(1, REGION_HEIGHT)-1
            )
        )
    }
    let mut subregion_idx = vec![0usize; (REGION_WIDTH * REGION_HEIGHT) as usize];
    for (idx,_) in hm.iter().enumerate() {
        let tile_loc = Point::new(
            idx % REGION_WIDTH as usize,
            idx / REGION_WIDTH as usize
        );

        let mut distance = std::f32::MAX;
        let mut sub_idx : usize = 0;
        for (c, center) in centroids.iter().enumerate() {
            let distance_from_centroid = DistanceAlg::Pythagoras.distance2d(tile_loc, *center);
            if distance_from_centroid < distance {
                distance = distance_from_centroid;
                sub_idx = c;
            }
        }
        subregion_idx[idx] = sub_idx;
    }

    // Sub-biomes
    let mut sb_variance = vec![0i32; n_subregions as usize];
    for sr in sb_variance.iter_mut() {
        let up_variance = rng.roll_dice(1, 2)-1;
        let down_variance = rng.roll_dice(1, 2)-1;
        let amount = up_variance - down_variance;
        *sr = amount;

        // Murky pools
        if rng.roll_dice(1, 500) < biome.mean_rainfall as i32 {
            *sr = -10;
        }
    }

    // Apply them
    for y in 0..REGION_HEIGHT {
        for x in 0..REGION_WIDTH {
            let tile_idx = ((y * REGION_WIDTH) + x) as usize;
            let sub_idx = subregion_idx[tile_idx];
            let delta_z = sb_variance[sub_idx];
            if DistanceAlg::Pythagoras.distance2d(Point::new(x,y), center_point) > 20.0 {
                if delta_z == -10 {
                    let h = hm[tile_idx] as i32;
                    hm[tile_idx] = (h + delta_z) as u8;
                    water[tile_idx] = h as u8-2;
                } else {
                    let h = hm[tile_idx] as i32;
                    hm[tile_idx] = (h + delta_z) as u8;
                }
            } else {
                // Ensure the crash site is clear
                hm[tile_idx] = hm[(( REGION_HEIGHT/2 * REGION_WIDTH) + ((REGION_WIDTH/2))) as usize];
                if hm[tile_idx] < 7 { hm[tile_idx] = 7; }
            }
        }
    }
}