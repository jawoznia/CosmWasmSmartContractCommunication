use cosmwasm_std::{Addr, Empty, Timestamp};
use cw_storage_plus::{Item, Map};

// with Item<Vec<T>> you need to load, modify, save -> this is gas costly
// Use map instead so that it will just save new vote without loading
// Every vote is accepting.
pub const VOTES: Map<Addr, Empty> = Map::new("votes");
pub const REQUIRED_APPROVALS: Item<u32> = Item::new("required_approvals");
pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
pub const ADMIN_CODE_ID: Item<u64> = Item::new("admin_code_id");
pub const ONGOING_VOTE: Item<Addr> = Item::new("ongoing_vote");
pub const START_TIME: Item<Timestamp> = Item::new("start_time");
