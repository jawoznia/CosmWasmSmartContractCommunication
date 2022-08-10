use crate::error::ContractError;
use crate::state::{Admin, ADMINS, DONATION_DENOM, VOTE_CODE_ID};
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use msgs::admin::{AdminsListResp, ExecuteMsg, InstantiateMsg, JoinTimeResp, QueryMsg};
use msgs::vote::InstantiateMsg as VoteInstantiate;

pub const VOTE_INSTANTIATE_ID: u64 = 1;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins: StdResult<Vec<Admin>> = msg
        .admins
        .into_iter()
        .map(|addr| -> StdResult<Admin> {
            Ok(Admin::new(deps.api.addr_validate(&addr)?, env.block.time))
        })
        .collect();
    ADMINS.save(deps.storage, &admins?)?;
    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;
    VOTE_CODE_ID.save(deps.storage, &msg.vote_code_id)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        AdminsList {} => to_binary(&query::admins_list(deps)?),
        JoinTime { admin } => to_binary(&query::join_time(deps, admin)?),
    }
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddMember {} => exec::add_member(deps, env, info),
        ProposeAdmin {
            addr,
            required_votes,
            admin_code_id,
        } => exec::propose_admin(deps, info, addr, required_votes, admin_code_id),
        Leave {} => exec::leave(deps, info).map_err(Into::into),
        Donate {} => exec::donate(deps, info),
    }
}

pub mod exec {
    use cosmwasm_std::Addr;
    use cosmwasm_std::SubMsg;
    use cosmwasm_std::SubMsgResult;
    use cw_utils::parse_instantiate_response_data;

    use crate::state::vote::PROPOSED_ADMIN;
    use crate::state::PENDING_VOTES;

    use super::*;
    use cosmwasm_std::WasmMsg;

    pub fn add_member(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut curr_admins = ADMINS.load(deps.storage)?;

        let proposed_admin = PENDING_VOTES.load(deps.storage, info.sender)?;

        if curr_admins
            .iter()
            .map(|admin| admin.addr().as_str())
            .any(|x| x == proposed_admin.as_str())
        {
            return Ok(Response::new());
        }

        curr_admins.push(Admin::new(proposed_admin, env.block.time));
        ADMINS.save(deps.storage, &curr_admins)?;

        Ok(Response::new())
    }

    pub fn propose_admin(
        deps: DepsMut,
        info: MessageInfo,
        addr: String,
        required_votes: u32,
        admin_code_id: u64,
    ) -> Result<Response, ContractError> {
        let msg = VoteInstantiate {
            required_votes,
            proposed_admin: addr,
            admin_code_id,
        };

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: VOTE_CODE_ID.load(deps.storage)?,
            msg: to_binary(&msg)?,
            funds: vec![],
            label: format!("admin-{}", info.sender),
        };

        let resp = Response::new()
            .add_submessage(SubMsg::reply_on_success(msg, VOTE_INSTANTIATE_ID))
            .add_attribute("action", "propose_admin")
            .add_attribute("sender", info.sender);

        Ok(resp)
    }

    pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        ADMINS.update(deps.storage, move |admins| -> StdResult<_> {
            let admins = admins
                .into_iter()
                .filter(|admin| *admin.addr() != info.sender)
                .collect();
            Ok(admins)
        })?;

        Ok(Response::new())
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let denom = DONATION_DENOM.load(deps.storage)?;
        let admins = ADMINS.load(deps.storage)?;

        let donation = cw_utils::must_pay(&info, &denom)
            .map_err(|err| StdError::generic_err(err.to_string()))?
            .u128();

        let donation_per_admin = donation / (admins.len() as u128);

        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.addr().to_string(),
            amount: coins(donation_per_admin, &denom),
        });

        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("action", "donate")
            .add_attribute("amount", donation.to_string())
            .add_attribute("per_admin", donation_per_admin.to_string());

        Ok(resp)
    }

    pub fn vote_instantiate_reply(deps: DepsMut, msg: SubMsgResult) -> StdResult<Response> {
        let resp = match msg.into_result() {
            Ok(resp) => resp,
            Err(err) => return Err(StdError::generic_err(err)),
        };

        let data = resp
            .data
            .ok_or_else(|| StdError::generic_err("No instantiate response data"))?;
        let resp = parse_instantiate_response_data(&data)
            .map_err(|err| StdError::generic_err(err.to_string()))?;
        let vote_addr = Addr::unchecked(&resp.contract_address);

        let proposed_admin = PROPOSED_ADMIN.query(&deps.querier, vote_addr.clone())?;
        PENDING_VOTES.save(deps.storage, vote_addr.clone(), &proposed_admin)?;

        let resp = Response::new().set_data(to_binary(&vote_addr)?);
        Ok(resp)
    }
}

mod query {
    use super::*;

    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let admins = ADMINS
            .load(deps.storage)?
            .into_iter()
            .map(|admin| admin.addr().clone())
            .collect();
        let resp = AdminsListResp { admins };
        Ok(resp)
    }

    pub fn join_time(deps: Deps, addr: String) -> StdResult<JoinTimeResp> {
        let admin = ADMINS
            .load(deps.storage)?
            .into_iter()
            .find(|admin| *admin.addr() == addr)
            .ok_or_else(|| StdError::generic_err("Admin not found!"))?;
        let resp = JoinTimeResp {
            joined: *admin.ts(),
        };
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::reply;

    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use contract_vote::execute as vote_execute;
    use contract_vote::instantiate as vote_instantiate;
    use contract_vote::query as vote_query;

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
                    admins: vec![],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: VOTE_INSTANTIATE_ID,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp, AdminsListResp { admins: vec![] });

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: VOTE_INSTANTIATE_ID,
                },
                &[],
                "Contract 2",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(
            resp,
            AdminsListResp {
                admins: vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")],
            }
        );
    }

    #[test]
    fn donations() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(5, "eth"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: VOTE_INSTANTIATE_ID,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "eth"),
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            0
        );

        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            1
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin1", "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );

        assert_eq!(
            app.wrap()
                .query_balance("admin2", "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );
    }

    #[test]
    fn propose_admin() {
        let admin_code_id = 1;
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        let code_id = app.store_code(Box::new(code));

        let vote_code = ContractWrapper::new(vote_execute, vote_instantiate, vote_query);
        let vote_code_id = app.store_code(Box::new(vote_code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    admins: vec![],
                    donation_denom: "eth".to_owned(),
                    vote_code_id: vote_code_id,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp, AdminsListResp { admins: vec![] });

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::ProposeAdmin {
                addr: String::from("proposed_admin"),
                required_votes: 2,
                admin_code_id,
            },
            &[],
        )
        .unwrap();
    }
}
