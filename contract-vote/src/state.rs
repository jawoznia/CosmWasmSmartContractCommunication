use cosmwasm_std::{Addr, Empty, Timestamp};
use cw_storage_plus::{Item, Map};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
// Every vote is accepting.
pub const VOTES: Map<Addr, Empty> = Map::new("votes");
pub const REQUIRED_APPROVALS: Item<u32> = Item::new("required_approvals");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
pub const VOTE_OWNER: Item<Addr> = Item::new("vote_owner");
pub const START_TIME: Item<Timestamp> = Item::new("start_time");

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Getters)]
#[serde(rename_all = "snake_case")]
pub struct Admin {
    addr: Addr,
    ts: Timestamp,
}

impl Admin {
    pub fn new(addr: Addr, ts: Timestamp) -> Admin {
        Admin { addr, ts }
    }
}

pub const ADMINS: Item<Vec<Admin>> = Item::new("admins");
