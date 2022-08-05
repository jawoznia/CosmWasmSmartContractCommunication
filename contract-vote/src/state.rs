use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
// Every vote is accepting.
pub const VOTES: Map<Addr, Empty> = Map::new("votes");
pub const REQUIRED_APPROVALS: Item<u32> = Item::new("required_approvals");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
