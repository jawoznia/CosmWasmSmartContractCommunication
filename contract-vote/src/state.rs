use cosmwasm_std::{Addr, Empty, Timestamp};
use cw_storage_plus::{Item, Map};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
// Every vote is accepting.
pub const VOTES: Map<Addr, Empty> = Map::new("votes");
pub const REQUIRED_VOTES: Item<u32> = Item::new("required_approvals");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
pub const VOTE_OWNER: Item<Addr> = Item::new("vote_owner");
pub const START_TIME: Item<Timestamp> = Item::new("start_time");

pub mod admin {
    use cosmwasm_std::Decimal;

    use super::*;
    pub const ADMINS: Map<Addr, Timestamp> = Map::new("admins");
    pub const QUORUM: Item<Decimal> = Item::new("quorum");
}
