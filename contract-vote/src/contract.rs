use std::ops::Add;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{NEEDED_VOTES_LEFT, VOTES};
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError,
    StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<()> {
    NEEDED_VOTES_LEFT.save(deps.storage, &msg.required)?;
    Ok(())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<()> {
    use QueryMsg::*;
    Ok(())
}

pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<()> {
    use ExecuteMsg::*;

    match msg {
        AddVote { approved } => {
            if VOTES.has(deps.storage, info.sender.clone()) {
                return Err(StdError::generic_err(format!(
                    "{} has already voted!",
                    &info.sender
                )));
            }
            NEEDED_VOTES_LEFT.update(deps.storage, |votes_left: u32| -> StdResult<u32> {
                Ok(votes_left - 1)
            })?;

            VOTES.save(deps.storage, info.sender, &approved)?;
        }
        _ => return Err(StdError::generic_err("Unimplemented")),
    }
    Ok(())
}
