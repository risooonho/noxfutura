use super::*;
use crate::prelude::*;
use crate::WorkOrder;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MyTurn {
    pub active: bool,
    pub shift: ScheduleTime,
    pub job: JobType,
    pub order: WorkOrder,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JobType {
    None,
    CollectTool {
        tool_id: usize,
        step: CollectToolSteps,
    },
    Haul {
        item_id: usize,
        step: HaulSteps,
    },
    FellTree {
        tool_id: Option<usize>,
        step: LumberjackSteps,
    },
    ConstructBuilding {
        building_id: usize,
        step: BuildingSteps,
    },
    Mining {
        step: MiningSteps,
        tool_id: Option<usize>,
    },
    Reaction {
        workshop_id: usize,
        workshop_pos: usize,
        reaction_id: usize,
        components: Vec<(usize, usize, bool, usize)>, // id, pos, claim, material
        step: ReactionSteps,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum CollectToolSteps {
    TravelToTool { path: Vec<usize> },
    CollectTool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum HaulSteps {
    FindItem,
    TravelToItem { path: Vec<usize> },
    CollectItem,
    TravelToDestination { path: Vec<usize> },
    DropItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LumberjackSteps {
    FindAxe,
    FindTree,
    ChopTree,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MiningSteps {
    FindPick,
    TravelToMine,
    Dig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BuildingSteps {
    FindBuilding,
    TravelToBuilding { path: Vec<usize> },
    Construct,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ReactionSteps {
    ClaimEverything,
    FindComponent,
    TravelToComponent {
        path: Vec<usize>,
        component_id: usize,
    },
    CollectComponent {
        component_id: usize,
    },
    FindWorkshop {
        component_id: usize,
    },
    TravelToWorkshop {
        path: Vec<usize>,
        component_id: usize,
    },
    Construct,
}
