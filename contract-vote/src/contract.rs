use crate::msg::{ApprovingAdminsResp, InstantiateMsg, QueryMsg, VotesLeftResp};
use crate::state::{PROPOSED_ADMIN, REQUIRED_APPROVALS};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    REQUIRED_APPROVALS.save(deps.storage, &msg.required)?;
    PROPOSED_ADMIN.save(deps.storage, &msg.proposed_admin)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        ApprovingAdmins {} => to_binary(&query::approving_admins()?),
        VotesLeft {} => to_binary(&query::votes_left(deps)?),
        ProposedAdmin {} => to_binary(&query::proposed_admin(deps)?),
    }
}

mod query {
    use crate::{msg::ProposedAdminResp, state::PROPOSED_ADMIN};

    use super::*;

    pub fn approving_admins() -> StdResult<ApprovingAdminsResp> {
        todo!()
    }

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
    use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response, StdResult};

    use crate::{
        msg::AcceptMsg,
        state::{REQUIRED_APPROVALS, VOTES},
    };

    pub fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _msg: AcceptMsg,
    ) -> StdResult<Response> {
        if VOTES.has(deps.storage, info.sender.clone()) {
            return Ok(Response::new());
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
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        contract::exec::execute,
        msg::{AcceptMsg, ProposedAdminResp},
    };

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

        let code = ContractWrapper::new(execute, instantiate, query);
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
