#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, attr, to_binary};

use crate::taxation::{deduct_tax};
use crate::state::{Config, StateData, read_config, store_config, read_state, store_state};
use crate::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use terraswap::querier::{query_balance};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: _info.sender,
            angel_addr:  deps.api.addr_validate(&msg.angel_addr)?,
            operation_addr: deps.api.addr_validate(&msg.operation_addr)?,
            reward_factor_operation: msg.reward_factor_operation,
            collector_addr: msg.collector_addr,
            reward_factor_collector: msg.reward_factor_collector
        },
    )?;

    store_state(
        deps.storage,
        &StateData{
            accumulated_angel_amount_ust: Uint128::zero(),
            accumulated_angel_amount_luna: Uint128::zero(),
        }
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            collector_addr,
            angel_addr,
            operation_addr,
            reward_factor_operation
        } => update_config(deps, info, owner, angel_addr, collector_addr, operation_addr, reward_factor_operation),
        ExecuteMsg::Distribute { denom } => distribute(deps,info, env, denom),
        ExecuteMsg::Withdraw { denom, addr } => withdraw(deps,info, env, denom, addr)
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    angel_addr: Option<String>,
    collector_addr: Option<String>,
    operation_addr: Option<String>,
    reward_factor_operation: Option<Decimal>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(angel_addr) = angel_addr {
        config.angel_addr = deps.api.addr_validate(&angel_addr)?;
    }

    if let Some(collector_addr) = collector_addr {
        config.collector_addr = deps.api.addr_validate(&collector_addr)?;
    }

    if let Some(operation_addr) = operation_addr {
        config.operation_addr = deps.api.addr_validate(&operation_addr)?;
    }

    if let Some(reward_factor_operation) = reward_factor_operation {
        config.reward_factor_operation = reward_factor_operation;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::default())
}

pub fn withdraw(deps: DepsMut,info: MessageInfo, env: Env, denom: String, addr: String) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let amount = query_balance(&deps.querier, env.contract.address, denom.to_string())?;
    let receiver = deps.api.addr_validate(&addr)?;

    let msg = BankMsg::Send {
        to_address: receiver.to_string(),
        amount: vec![deduct_tax(
           &deps.querier,
           Coin {
               denom: denom.clone(),
               amount: amount,
           },
       )?],
    };

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Bank(msg)
        ])
        .add_attributes(vec![
            attr("action", "withdraw"),
            attr("denom", denom.clone()),
            attr("amount", amount.clone().to_string()),
        ]))
}

/// distribute
pub fn distribute(deps: DepsMut,info: MessageInfo, env: Env, denom: String) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    let mut state: StateData = read_state(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let amount = query_balance(&deps.querier, env.contract.address, denom.to_string())?;
    // one part will send to operation wallet, another send to angel
    let operations_amount = amount * config.reward_factor_operation;
    let collector_amount = amount * config.reward_factor_collector;
    let angel_amount = amount - operations_amount - collector_amount;

    let oper_msg = BankMsg::Send {
        to_address: config.operation_addr.to_string(),
        amount: vec![deduct_tax(
           &deps.querier,
           Coin {
               denom: denom.clone(),
               amount: operations_amount,
           },
       )?],
    };

    let collector_msg = BankMsg::Send {
        to_address: config.collector_addr.to_string(),
        amount: vec![deduct_tax(
           &deps.querier,
           Coin {
               denom: denom.clone(),
               amount: collector_amount,
           },
       )?],
    };


    let angel_msg = BankMsg::Send {
        to_address: config.angel_addr.to_string(),
        amount: vec![deduct_tax(
           &deps.querier,
           Coin {
               denom: denom.clone(),
               amount: angel_amount,
           },
       )?],
    };

    if denom == "uusd" {
        state.accumulated_angel_amount_ust += angel_amount;
    } else if denom == "uluna" {
        state.accumulated_angel_amount_luna += angel_amount;
    }

    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Bank(oper_msg),
            CosmosMsg::Bank(collector_msg),
            CosmosMsg::Bank(angel_msg),
        ])
        .add_attributes(vec![
            attr("action", "distribute"),
            attr("denom", denom.clone()),
            attr("distribute_operations_amount", operations_amount.clone().to_string()),
            attr("collector_msg", collector_amount.clone().to_string()),
            attr("angel_amount", angel_amount.clone().to_string()),
        ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: config.owner,
        angel_addr: config.angel_addr,
        operation_addr: config.operation_addr,
        reward_factor_operation: config.reward_factor_operation,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps) -> StdResult<StateData> {
    let state = read_state(deps.storage)?;
    Ok(state)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
