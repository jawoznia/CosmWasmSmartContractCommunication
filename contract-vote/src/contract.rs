use crate::state::{PROPOSED_ADMIN, REQUIRED_APPROVALS, START_TIME};
use contract_msgs::vote::{InstantiateMsg, QueryMsg, VotesLeftResp};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    REQUIRED_APPROVALS.save(deps.storage, &msg.required)?;
    PROPOSED_ADMIN.save(deps.storage, &msg.proposed_admin)?;
    START_TIME.save(deps.storage, &env.block.time)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        VotesLeft {} => to_binary(&query::votes_left(deps)?),
        ProposedAdmin {} => to_binary(&query::proposed_admin(deps)?),
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
    use cosmwasm_std::{
        from_binary, to_binary, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
        SubMsg, SubMsgResult, WasmMsg,
    };

    use crate::state::{ADMIN_CODE_ID, ONGOING_VOTE, REQUIRED_APPROVALS, START_TIME, VOTES};
    use contract_msgs::{admin::JoinTimeResp, vote::AcceptMsg};

    pub const ADMIN_JOIN_TIME_QUERY_ID: u64 = 1;

    pub fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _msg: AcceptMsg,
    ) -> StdResult<Response> {
        if VOTES.has(deps.storage, info.sender.clone()) {
            return Ok(Response::new());
        }
        ONGOING_VOTE.save(deps.storage, &info.sender)?;

        let msg = contract_msgs::admin::QueryMsg::JoinTime {
            admin: info.sender.clone(),
        };

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: ADMIN_CODE_ID.load(deps.storage)?,
            msg: to_binary(&msg)?,
            funds: vec![],
            label: format!("peer-{}", info.sender),
        };

        let resp = Response::new()
            .add_submessage(SubMsg::reply_on_success(msg, ADMIN_JOIN_TIME_QUERY_ID))
            .add_attribute("action", "propose_admin")
            .add_attribute("sender", info.sender);

        Ok(resp)
    }

    pub fn admin_join_time_reply(deps: DepsMut, msg: SubMsgResult) -> StdResult<Response> {
        let resp = match msg.into_result() {
            Ok(resp) => resp,
            Err(err) => return Err(StdError::generic_err(err)),
        };

        let tmp = resp.data.ok_or_else(|| StdError::generic_err("blabla"))?;

        let tmp: JoinTimeResp = from_binary(&tmp)?;

        if tmp.joined < START_TIME.load(deps.storage)? {
            return Err(StdError::generic_err(
                "Admin is not allowed to vote due to being approved after vote is created.",
            ))?;
        }

        REQUIRED_APPROVALS.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
            Ok(votes_left - 1)
        })?;

        let empty_value = Empty {};
        let sender_addr = ONGOING_VOTE.load(deps.storage)?;
        VOTES.save(deps.storage, sender_addr, &empty_value)?;

        Ok(Response::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::exec::execute;
    use crate::reply;
    use contract_msgs::vote::{AcceptMsg, ProposedAdminResp};
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

    #[test]
    fn accept() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    proposed_admin: Addr::unchecked("new_admin"),
                    required: 3,
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

        app.execute_contract(Addr::unchecked("admin1"), addr.clone(), &AcceptMsg {}, &[])
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        app.execute_contract(Addr::unchecked("admin1"), addr.clone(), &AcceptMsg {}, &[])
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 2 });

        app.execute_contract(Addr::unchecked("admin2"), addr.clone(), &AcceptMsg {}, &[])
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 1 });

        app.execute_contract(Addr::unchecked("admin3"), addr.clone(), &AcceptMsg {}, &[])
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 0 });

        app.execute_contract(Addr::unchecked("admin3"), addr.clone(), &AcceptMsg {}, &[])
            .unwrap();

        let resp: VotesLeftResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeft {})
            .unwrap();

        assert_eq!(resp, VotesLeftResp { votes_left: 0 });
    }
}
