use crate::state::{PROPOSED_ADMIN, REQUIRED_VOTES, START_TIME, VOTE_OWNER};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use msgs::vote::{ExecuteMsg, InstantiateMsg, QueryMsg, VotesLeftResp};

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    REQUIRED_VOTES.save(deps.storage, &msg.required_votes)?;
    PROPOSED_ADMIN.save(deps.storage, &deps.api.addr_validate(&msg.proposed_admin)?)?;
    START_TIME.save(deps.storage, &env.block.time)?;
    VOTE_OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        VotesLeft {} => to_binary(&query::votes_left(deps)?),
        ProposedAdmin {} => to_binary(&query::proposed_admin(deps)?),
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
        to_binary, DepsMut, Empty, MessageInfo, Response, StdError, StdResult, SubMsg, WasmMsg,
    };
    use msgs::admin::ExecuteMsg;

    use crate::state::{admin::ADMINS, REQUIRED_VOTES, START_TIME, VOTES, VOTE_OWNER};

    pub const ADMIN_JOIN_TIME_QUERY_ID: u64 = 1;

    pub fn accept(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        if VOTES.has(deps.storage, info.sender.clone()) {
            return Ok(Response::new());
        }

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

        REQUIRED_VOTES.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
            Ok(votes_left - 1)
        })?;

        VOTES.save(deps.storage, info.sender, &Empty {})?;

        if REQUIRED_VOTES.load(deps.storage)? > 0 {
            return Ok(Response::new()
                .add_attribute("action", "accept")
                .add_attribute("status", "Voting has already passed."));
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
}

#[cfg(test)]
mod tests {
    use crate::contract::execute;

    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};
    use msgs::vote::ProposedAdminResp;

    use super::*;

    #[test]
    fn instantiation() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    proposed_admin: String::from("proposed_admin"),
                    required_votes: 3,
                    admin_code_id: code_id,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 3 });

        let resp: ProposedAdminResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::ProposedAdmin {})
            .unwrap();

        assert_eq!(
            resp,
            ProposedAdminResp {
                proposed_admin: Addr::unchecked("proposed_admin")
            }
        );
    }
}
