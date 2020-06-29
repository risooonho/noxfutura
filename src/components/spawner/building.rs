use crate::components::*;
use legion::prelude::*;

pub fn spawn_building(ecs: &mut World, tag: &str, x: usize, y: usize, z: usize) {
    use crate::raws::*;
    let rlock = crate::raws::RAWS.read();
    if let Some(building_def) = rlock.buildings.building_by_tag(tag) {
        let dims = if let Some(dims) = building_def.dimensions {
            Dimensions {
                width: dims.0 as i32,
                height: dims.1 as i32,
            }
        } else {
            Dimensions {
                width: 1,
                height: 1,
            }
        };

        let entity = ecs.insert(
            (Building {},),
            vec![(
                Identity::new(),
                Name {
                    name: building_def.name.clone(),
                },
                dims,
                crate::components::VoxelModel {
                    index: rlock.vox.get_model_idx(&building_def.vox),
                },
                Description {
                    desc: building_def.description.clone(),
                },
                Position { x, y, z },
                Tint {
                    color: (1.0, 1.0, 1.0),
                },
            )],
        )[0].clone();

        for provides in building_def.provides.iter() {
            if let BuildingProvides::Light{radius, color} = provides {
                ecs.add_component(entity, Light{
                    color: *color, radius: *radius
                }).expect("Unable to add light");
                ecs.add_component(entity, FieldOfView::new(*radius) ).expect("Unable to add field-of-view");
            }
        }

        println!("Added building data: {}", tag);
    } else {
        println!("Failed to spawn building: {}", tag);
    }
}