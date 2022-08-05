use std::ops::Add;

use crate::msg::{
    ApprovingAdminsResp, ExecuteMsg, InstantiateMsg, QueryMsg, VotesLeftForApprovalResp,
};
use crate::state::{NEEDED_APPROVALS_LEFT, PROPOSED_ADMIN, VOTES};
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response,
    StdError, StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    NEEDED_APPROVALS_LEFT.save(deps.storage, &msg.required)?;
    PROPOSED_ADMIN.save(deps.storage, &msg.proposed_admin)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        ApprovingAdmins {} => to_binary(&query::approving_admins()?),
        VotesLeftForApproval {} => to_binary(&query::votes_left(deps)?),
        ProposedAdmin {} => to_binary(&query::proposed_admin(deps)?),
    }
}

mod query {

    use crate::{msg::ProposedAdminResp, state::PROPOSED_ADMIN};

    use super::*;

    pub fn approving_admins() -> StdResult<ApprovingAdminsResp> {
        todo!()
    }

    pub fn votes_left(deps: Deps) -> StdResult<VotesLeftForApprovalResp> {
        let resp = VotesLeftForApprovalResp {
            votes_left: NEEDED_APPROVALS_LEFT.load(deps.storage)?,
        };
        Ok(resp)
    }

    pub fn proposed_admin(deps: Deps) -> StdResult<ProposedAdminResp> {
        let resp = ProposedAdminResp {
            admin: PROPOSED_ADMIN.load(deps.storage)?,
        };
        Ok(resp)
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        Accept => {
            if VOTES.has(deps.storage, info.sender.clone()) {
                // return Err(StdError::generic_err(format!(
                //     "{} has already voted!",
                //     &info.sender
                // )));
                return Ok(Response::new())
            }
            NEEDED_APPROVALS_LEFT.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
                Ok(votes_left - 1)
            })?;

            let empty_value = Empty {};
            VOTES.save(deps.storage, info.sender, &empty_value)?;

            Ok(Response::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::msg::ProposedAdminResp;

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
                    proposed_admin: Addr::unchecked("new_admin"),
                    required: 3,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 3 });

        let resp: ProposedAdminResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::ProposedAdmin {})
            .unwrap();

        assert_eq!(
            resp,
            ProposedAdminResp {
                admin: Addr::unchecked("new_admin")
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

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 3 });

        app.execute_contract(
            Addr::unchecked("admin1"),
            addr.clone(),
            &ExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 2 });

        app.execute_contract(
            Addr::unchecked("admin1"),
            addr.clone(),
            &ExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 2 });

        app.execute_contract(
            Addr::unchecked("admin2"),
            addr.clone(),
            &ExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 1 });

        app.execute_contract(
            Addr::unchecked("admin3"),
            addr.clone(),
            &ExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 0 });

        app.execute_contract(
            Addr::unchecked("admin3"),
            addr.clone(),
            &ExecuteMsg::Accept {},
            &[],
        )
        .unwrap();

        let resp: VotesLeftForApprovalResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::VotesLeftForApproval {})
            .unwrap();

        assert_eq!(resp, VotesLeftForApprovalResp { votes_left: 0 });
    }
}
