#![deny(warnings)]

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct IncrementReq {
    pub increment_counter_by: u32,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct IncrementResp {
    pub new_counter_state: u32,
}
