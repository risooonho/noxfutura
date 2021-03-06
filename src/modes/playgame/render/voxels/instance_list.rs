use super::VoxBuffer;
use bengine::Palette;
use bengine::*;
use legion::*;
use nox_components::*;
use nox_utils::Frustrum;
use std::collections::HashMap;

const FRUSTRUM_CHECK_RANGE: f32 = 2.0;

#[derive(Debug)]
pub struct VMRender {
    position: [f32; 3],
    tint: usize,
    rotation: f32,
    greyscale: f32,
}

#[derive(Debug)]
pub struct VMInstances {
    instances: HashMap<usize, Vec<VMRender>>,
}

impl VMInstances {
    pub fn new() -> Self {
        Self {
            instances: HashMap::with_capacity(200),
        }
    }

    fn add(
        &mut self,
        model_id: usize,
        position: [f32; 3],
        tint: usize,
        rotation: f32,
        greyscale: f32,
    ) {
        if let Some(vmi) = self.instances.get_mut(&model_id) {
            vmi.push(VMRender {
                position,
                tint,
                rotation,
                greyscale,
            });
        } else {
            self.instances.insert(
                model_id,
                vec![VMRender {
                    position,
                    tint,
                    rotation,
                    greyscale,
                }],
            );
        }
    }
}

const LAYERS_DOWN: usize = 50;

pub fn build_vox_instances2(
    ecs: &World,
    camera_z: usize,
    vox_models: &VoxBuffer,
    instance_buffer: &mut FloatBuffer<f32>,
    vox_instances: &mut Vec<(u32, u32, u32)>,
    frustrum: &Frustrum,
    mouse_world_pos: &(usize, usize, usize),
    building_to_build: &Option<usize>,
    palette: &Palette,
) {
    let mut instances = VMInstances::new();
    instance_buffer.clear();
    vox_instances.clear();

    // Models from the ECS
    let mut query = <(Entity, Read<Position>, Read<VoxelModel>, Read<Tint>)>::query();
    query
        .iter(ecs)
        .filter(|(_, pos, _, _)| {
            if let Some(pt) = pos.as_point3_only_tile() {
                pt.z as usize > camera_z - LAYERS_DOWN
                    && pt.z as usize <= camera_z
                    && frustrum.check_sphere(&pos.as_vec3(), FRUSTRUM_CHECK_RANGE)
            } else {
                false
            }
        })
        .for_each(|(entity, pos, model, tint)| {
            let mut pt = pos.as_vec3();

            if pos.dimensions.0 == 3 {
                pt.x -= 1.0;
            }
            if pos.dimensions.1 == 3 {
                pt.y -= 1.0;
            }

            instances.add(
                model.index,
                [pt.x, pt.z, pt.y],
                tint.color,
                model.rotation_radians,
                if let Ok(b) = ecs.entry_ref(*entity).unwrap().get_component::<Building>() {
                    if b.complete {
                        0.0
                    } else {
                        1.0
                    }
                } else {
                    0.0
                },
            );
        });

    // Composite builder
    let mut query = <(Read<Position>, Read<CompositeRender>)>::query();
    query
        .iter(ecs)
        .filter(|(pos, _)| {
            if let Some(pt) = pos.as_point3_only_tile() {
                pt.z as usize > camera_z - LAYERS_DOWN
                    && pt.z as usize <= camera_z
                    && frustrum.check_sphere(
                        &(pt.x as f32, pt.y as f32, pt.z as f32).into(),
                        FRUSTRUM_CHECK_RANGE,
                    )
            } else {
                false
            }
        })
        .for_each(|(pos, composite)| {
            for vm in composite.layers.iter() {
                instances.add(
                    vm.model,
                    pos.as_xzy_f32(),
                    palette.find_palette(vm.tint.0, vm.tint.1, vm.tint.2),
                    composite.rotation,
                    0.0,
                );
            }
        });

    // Building Projects
    if let Some(tag) = building_to_build {
        instances.add(
            *tag,
            [
                mouse_world_pos.0 as f32,
                mouse_world_pos.2 as f32,
                mouse_world_pos.1 as f32,
            ],
            palette.find_palette(0.5, 0.5, 0.5),
            0.0,
            1.0,
        );
    }

    // Stairs from terrain
    let region = crate::modes::playgame::systems::REGION.read();
    use nox_planet::{StairsType, TileType};
    region
        .tile_types
        .iter()
        .enumerate()
        .filter(|(_idx, tt)| match tt {
            TileType::Stairs { .. } => true,
            _ => false,
        })
        .map(|(idx, tt)| {
            let (x, y, z) = nox_spatial::idxmap(idx);
            (tt, x as f32, y as f32, z as f32)
        })
        .filter(|(_tt, x, y, z)| {
            *z as usize > camera_z - LAYERS_DOWN
                && *z as usize <= camera_z
                && frustrum.check_sphere(&(*x, *y, *z).into(), 2.0)
        })
        .for_each(|(tt, x, y, z)| match tt {
            TileType::Stairs {
                direction: StairsType::Up,
            } => {
                let model_id = nox_raws::RAWS.read().vox.get_model_idx("stairs_up");
                instances.add(model_id, [x as f32, z as f32, y as f32], 0, 0.0, 0.0);
            }
            TileType::Stairs {
                direction: StairsType::Down,
            } => {
                let model_id = nox_raws::RAWS.read().vox.get_model_idx("stairs_down");
                instances.add(model_id, [x as f32, z as f32, y as f32], 0, 0.0, 0.0);
            }
            TileType::Stairs {
                direction: StairsType::UpDown,
            } => {
                let model_id = nox_raws::RAWS.read().vox.get_model_idx("stairs_updown");
                instances.add(model_id, [x as f32, z as f32, y as f32], 0, 0.0, 0.0);
            }
            _ => {}
        });

    // Build the instanced data
    instances.instances.iter().for_each(|i| {
        vox_instances.push((
            vox_models.offsets[*i.0].0,
            vox_models.offsets[*i.0].1,
            i.1.len() as u32,
        ));
        i.1.iter().for_each(|vm| {
            instance_buffer.add_slice(&vm.position);
            instance_buffer.add(vm.tint as f32);
            instance_buffer.add(vm.rotation);
            instance_buffer.add(vm.greyscale);
        });
    });

    // Push the buffer update
    if !vox_instances.is_empty() {
        instance_buffer.update_buffer();
    }
}
