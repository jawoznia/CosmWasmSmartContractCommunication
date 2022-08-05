use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
// Every vote is
pub const VOTES: Map<Addr, Empty> = Map::new("votes");
pub const NEEDED_APPROVALS_LEFT: Item<u32> = Item::new("needed_approvals_left");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
