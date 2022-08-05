use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");
pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const VOTE_CODE_ID: Item<u64> = Item::new("vote_code_id");
