use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{NEEDED_VOTES_LEFT, VOTES};
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<()> {
    Ok(())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<()> {
    use QueryMsg::*;
    Ok(())
}

pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<()> {
    use ExecuteMsg::*;
    Ok(())
}
