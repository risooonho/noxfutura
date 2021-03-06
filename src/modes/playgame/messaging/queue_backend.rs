use super::JobStep;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};

lazy_static! {
    pub static ref JOBS_QUEUE: Mutex<VecDeque<JobStep>> = Mutex::new(VecDeque::new());
}

lazy_static! {
    pub static ref MOVER_LIST: Mutex<HashMap<usize, (usize, usize, usize)>> =
        Mutex::new(HashMap::new());
}
