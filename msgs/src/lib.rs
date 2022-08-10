use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod vote {
    use super::*;

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InstantiateMsg {
        pub required_votes: u32,
        pub proposed_admin: String,
        pub admin_code_id: u64,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        Accept {},
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        VotesLeft {},
        ProposedAdmin {},
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct VotesLeftResp {
        pub votes_left: u32,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ProposedAdminResp {
        pub proposed_admin: Addr,
    }
}

pub mod admin {
    use cosmwasm_std::Timestamp;

    use super::*;

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InstantiateMsg {
        pub admins: Vec<String>,
        pub donation_denom: String,
        pub vote_code_id: u64,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        AddMember {},
        ProposeAdmin {
            addr: String,
            required_votes: u32,
            admin_code_id: u64,
        },
        // How admins know that there is a voting ongoing and they need to send Accept message
        // Blockchain does not inform users about that. This is purely done on f.e. discord.
        // I believe you can also watch messages on blockchain which can give you a hint about that.
        Leave {},
        Donate {},
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        Greet {},
        AdminsList {},
        JoinTime { admin: String },
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct JoinTimeResp {
        pub joined: Timestamp,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct ProposeAdminResp {
        pub vote_addr: Addr,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct GreetResp {
        pub message: String,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct AdminsListResp {
        pub admins: Vec<Addr>,
    }
}
