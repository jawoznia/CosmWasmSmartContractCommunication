use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
pub const VOTES: Map<Addr, bool> = Map::new("votes");
pub const NEEDED_VOTES_LEFT: Item<u32> = Item::new("needed_votes_left");
