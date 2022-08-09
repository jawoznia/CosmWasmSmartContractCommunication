use crate::state::{PROPOSED_ADMIN, REQUIRED_APPROVALS, START_TIME, VOTE_OWNER};
use contract_msgs::vote::{ExecuteMsg, InstantiateMsg, QueryMsg, VotesLeftResp};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    REQUIRED_APPROVALS.save(deps.storage, &msg.required)?;
    PROPOSED_ADMIN.save(deps.storage, &msg.proposed_admin)?;
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
    use contract_msgs::vote::ProposedAdminResp;

    use super::*;

    pub fn votes_left(deps: Deps) -> StdResult<VotesLeftResp> {
        let resp = VotesLeftResp {
            votes_left: REQUIRED_APPROVALS.load(deps.storage)?,
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

    use cosmwasm_std::{DepsMut, Empty, MessageInfo, Response, StdError, StdResult};

    use crate::state::{ADMINS, REQUIRED_APPROVALS, START_TIME, VOTES, VOTE_OWNER};

    pub const ADMIN_JOIN_TIME_QUERY_ID: u64 = 1;

    pub fn accept(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        if VOTES.has(deps.storage, info.sender.clone()) {
            return Ok(Response::new());
        }

        let admins = ADMINS.query(&deps.querier, VOTE_OWNER.load(deps.storage)?)?;
        let sender = &info.sender;

        let admin_start_time = *admins
            .into_iter()
            .find(|admin| admin.addr() == sender)
            .ok_or_else(|| StdError::generic_err("Sender admin not found in storage!"))?
            .ts();
        let vote_start_time = START_TIME.load(deps.storage)?;

        if admin_start_time < vote_start_time {
            return Err(StdError::generic_err(
                "Admin is not allowed to vote due to being approved after vote is created.",
            ))?;
        }

        REQUIRED_APPROVALS.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
            Ok(votes_left - 1)
        })?;

        let empty_value = Empty {};
        VOTES.save(deps.storage, info.sender, &empty_value)?;

        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::execute;

    use contract_msgs::vote::ProposedAdminResp;
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

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
                    proposed_admin: Addr::unchecked("proposed_admin"),
                    required: 3,
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
