use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub required: u32,
    pub voting_admins: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Accept {},
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    VotesLeftForApproval {},
    ApprovingAdmins {},
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VotesLeftForApprovalResp {
    pub votes_left: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApprovingAdminsResp {
    pub admins: Vec<Addr>,
}
