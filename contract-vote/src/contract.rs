use std::ops::Add;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{NEEDED_APPROVALS_LEFT, VOTES};
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
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    todo!()
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        AddVote => {
            if VOTES.has(deps.storage, info.sender.clone()) {
                return Err(StdError::generic_err(format!(
                    "{} has already voted!",
                    &info.sender
                )));
            }
            NEEDED_APPROVALS_LEFT.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
                Ok(votes_left - 1)
            })?;

            let empty_value = Empty {};
            VOTES.save(deps.storage, info.sender, &empty_value)?;

            Ok(Response::new())
        }
        _ => return Err(StdError::generic_err("Unimplemented")),
    }
}

// #[cfg(test)]
// mod tests {
//     use cosmwasm_std::Addr;
//     use cw_multi_test::{App, ContractWrapper, Executor};

//     use super::*;

//     #[test]
//     fn instantiation() {
//         let mut app = App::default();

//         let code = ContractWrapper::new(execute, instantiate, query);
//         let code_id = app.store_code(Box::new(code));

//         let addr = app
//             .instantiate_contract(
//                 code_id,
//                 Addr::unchecked("owner"),
//                 &InstantiateMsg {
//                     admins: vec![],
//                     donation_denom: "eth".to_owned(),
//                 },
//                 &[],
//                 "Contract",
//                 None,
//             )
//             .unwrap();

//         let resp: AdminsListResp = app
//             .wrap()
//             .query_wasm_smart(addr, &QueryMsg::AdminsList {})
//             .unwrap();

//         assert_eq!(resp, AdminsListResp { admins: vec![] });

//         let addr = app
//             .instantiate_contract(
//                 code_id,
//                 Addr::unchecked("owner"),
//                 &InstantiateMsg {
//                     admins: vec!["admin1".to_owned(), "admin2".to_owned()],
//                     donation_denom: "eth".to_owned(),
//                 },
//                 &[],
//                 "Contract 2",
//                 None,
//             )
//             .unwrap();

//         let resp: AdminsListResp = app
//             .wrap()
//             .query_wasm_smart(addr, &QueryMsg::AdminsList {})
//             .unwrap();

//         assert_eq!(
//             resp,
//             AdminsListResp {
//                 admins: vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")],
//             }
//         );
//     }
// }
