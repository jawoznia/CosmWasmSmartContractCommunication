use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::Item;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

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
pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const VOTE_CODE_ID: Item<u64> = Item::new("vote_code_id");
