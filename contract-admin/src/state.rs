use cosmwasm_std::{Addr, Decimal, Timestamp};
use cw_storage_plus::{Item, Map};

pub const ADMINS: Map<Addr, Timestamp> = Map::new("admins");
pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const VOTE_CODE_ID: Item<u64> = Item::new("vote_code_id");
// voting contract to proposed admins
pub const PENDING_VOTES: Map<Addr, Addr> = Map::new("pending_votes");
pub const QUORUM: Item<Decimal> = Item::new("quorum");

pub mod vote {
    use super::*;

    pub const VOTE_OWNER: Item<Addr> = Item::new("vote_owner");
    pub const PROPOSED_ADMIN: Item<Addr> = Item::new("proposed_admin");
}
