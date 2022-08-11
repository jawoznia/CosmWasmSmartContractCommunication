use crate::state::{PROPOSED_ADMIN, REQUIRED_VOTES, START_TIME, VOTE_OWNER};
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use msgs::vote::QueryMsg;
use msgs::{
    admin::{AdminsListResp, QueryMsg as AdminQueryMsg},
    vote::{ExecuteMsg, InstantiateMsg, VotesLeftResp},
};

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    PROPOSED_ADMIN.save(deps.storage, &deps.api.addr_validate(&msg.proposed_admin)?)?;
    START_TIME.save(deps.storage, &env.block.time)?;
    VOTE_OWNER.save(deps.storage, &info.sender)?;

    let vote_owner = &info.sender;
    let quorum = msg.quorum;

    let resp: AdminsListResp = deps
        .querier
        .query_wasm_smart(vote_owner, &AdminQueryMsg::AdminsList {})?;

    let admins_decimals = match Decimal::from_atomics(resp.admins.len() as u128, 0) {
        Ok(val) => val,
        Err(err) => return Err(StdError::generic_err(err.to_string())),
    };

    let required_votes = quorum * admins_decimals;

    REQUIRED_VOTES.save(deps.storage, &required_votes)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotesLeft {} => to_binary(&query::votes_left(deps)?),
        QueryMsg::ProposedAdmin {} => to_binary(&query::proposed_admin(deps)?),
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Accept {} => exec::accept(deps, info),
    }
}

mod query {
    use crate::state::PROPOSED_ADMIN;
    use msgs::vote::ProposedAdminResp;

    use super::*;

    pub fn votes_left(deps: Deps) -> StdResult<VotesLeftResp> {
        let resp = VotesLeftResp {
            votes_left: REQUIRED_VOTES.load(deps.storage)?,
        };
        Ok(resp)
    }

    pub fn proposed_admin(deps: Deps) -> StdResult<ProposedAdminResp> {
        let resp = ProposedAdminResp {
            proposed_admin: PROPOSED_ADMIN.load(deps.storage)?,
        };
        Ok(resp)
    }
}

pub mod exec {

    use std::cmp::Ordering;

    use cosmwasm_std::{
        to_binary, Decimal, DepsMut, Empty, MessageInfo, Response, StdError, StdResult, SubMsg,
        WasmMsg,
    };
    use msgs::admin::ExecuteMsg;

    use crate::state::{admin::ADMINS, REQUIRED_VOTES, START_TIME, VOTES, VOTE_OWNER};

    pub fn accept(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        if VOTES.has(deps.storage, info.sender.clone()) {
            return Ok(Response::new());
        }

        validate_admin_prove_to_vote(&deps, &info)?;

        REQUIRED_VOTES.update(deps.storage, |votes_left| -> StdResult<Decimal> {
            Ok(votes_left - Decimal::one())
        })?;

        VOTES.save(deps.storage, info.sender, &Empty {})?;

        if REQUIRED_VOTES.load(deps.storage)? >= Decimal::one() {
            return Ok(Response::new()
                .add_attribute("action", "accept")
                .add_attribute("status", "Some admins still need to accept the voting."));
        }

        let msg = WasmMsg::Execute {
            contract_addr: VOTE_OWNER.load(deps.storage)?.into_string(),
            msg: to_binary(&ExecuteMsg::AddMember {})?,
            funds: vec![],
        };

        let resp = Response::new()
            .add_submessage(SubMsg::new(msg))
            .add_attribute("action", "accept");

        Ok(resp)
    }

    fn validate_admin_prove_to_vote(deps: &DepsMut, info: &MessageInfo) -> StdResult<()> {
        let admin_start_time = match ADMINS.query(
            &deps.querier,
            VOTE_OWNER.load(deps.storage)?,
            info.sender.clone(),
        )? {
            Some(v) => v,
            None => return Err(StdError::generic_err("Non admin accept!.")),
        };

        let vote_start_time = START_TIME.load(deps.storage)?;

        if admin_start_time.cmp(&vote_start_time) == Ordering::Greater {
            return Err(StdError::generic_err(
                "Admin is not allowed to vote due to being approved after vote is created.",
            ))?;
        }
        Ok(())
    }
}
