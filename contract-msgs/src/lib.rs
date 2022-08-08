use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod vote {
    use super::*;

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InstantiateMsg {
        pub required: u32,
        pub proposed_admin: Addr,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct AcceptMsg {}

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
        AddMembers { admins: Vec<String> },
        ProposeAdmin { addr: Addr, required_votes: u32 },
        // How admins know that there is a voting ongoing and they need to send Accept message
        // Blockchain does not inform users about that. This is purely done on f.e. discord.
        // I believe you can also watch messages on blockchain which can give you a hint about that.
        Leave {},
        Donate {},
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
        pub joined: String,
    }
}
