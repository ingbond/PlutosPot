use cosmwasm_std::{
     to_binary, Api, Binary, CosmosMsg, Env, entry_point,
      StdResult,HumanAddr, Uint128,
     WasmMsg, MessageInfo, Response, DepsMut, Deps, Addr, SubMsg, attr
};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, HandleMsg, InstantiateMsg, IsClaimedResponse, LatestStageResponse, MerkleRootResponse,
    MigrateMsg, QueryMsg
};
use crate::state::{
    read_claimed, read_config, read_latest_stage, read_merkle_root, store_claimed, store_config,
    store_latest_stage, store_merkle_root, Config
};

use cw20::Cw20ExecuteMsg;
use hex;
use sha3::Digest;
use std::convert::TryInto;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            pluto_token: deps.api.addr_validate(&msg.pluto_token)?,
        },
    )?;

    let stage: u8 = 0;
    store_latest_stage(deps.storage, stage)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<Response, ContractError> {
    match msg {
        HandleMsg::UpdateConfig { owner } => update_config(deps, env, info, owner),
        HandleMsg::UpdateMerkleRoot { stage, merkle_root } => {
            update_merkle_root(deps, env, info, stage, merkle_root)
        }
        HandleMsg::RegisterMerkleRoot { merkle_root } => {
            register_merkle_root(deps, env, info, merkle_root)
        }
        HandleMsg::Claim {
            stage,
            amount,
            proof,
        } => claim(deps, env, info, stage, amount, proof),
    }
}

pub fn update_merkle_root(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage: u8,
    merkle_root: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if deps.api.addr_validate(&info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    store_merkle_root(deps.storage, stage, merkle_root.to_string())?;

    let res = Response::new()
        .add_attribute("action", "update_merkle_root")
        .add_attribute("stage", stage.to_string())
        .add_attribute("merkle_root", merkle_root);

    Ok(res)
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<HumanAddr>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;
    if deps.api.addr_validate(&info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    store_config(deps.storage, &config)?;

    let res = Response::new()
        .add_attribute("action", "update_config");
    Ok(res)
}

pub fn register_merkle_root(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    merkle_root: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    if deps.api.addr_validate(&info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let latest_stage: u8 = read_latest_stage(deps.storage)?;
    let stage = latest_stage + 1;

    store_merkle_root(deps.storage, stage, merkle_root.to_string())?;
    store_latest_stage(deps.storage, stage)?;

    let res = Response::new()
        .add_attribute("action", "register_merkle_root")
        .add_attribute("stage", stage.to_string())
        .add_attribute("merkle_root", merkle_root)
        ;

    Ok(res)
}

pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage: u8,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;
    let merkle_root: String = read_merkle_root(deps.storage, stage)?;

    let user_raw = deps.api.addr_validate(&info.sender.as_str())?;

    // If user claimed target stage, return err
    if read_claimed(deps.storage, &user_raw, stage)? {
        return Err(ContractError::Claimed {});
    }

    let user_input: String = info.sender.to_string() + &amount.to_string();
    let mut hash: [u8; 32] = sha3::Keccak256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong length");

    for p in proof {
        let mut proof_buf: [u8; 32] = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf).unwrap();
        hash = if bytes_cmp(hash, proof_buf) == std::cmp::Ordering::Less {
            sha3::Keccak256::digest(&[hash, proof_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        } else {
            sha3::Keccak256::digest(&[proof_buf, hash].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        };
    }

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf).unwrap();
    if root_buf != hash {
        return Err(ContractError::Unauthorized {});
    }

    // Update claim index to the current stage
    store_claimed(deps.storage, &user_raw, stage)?;

    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_validate(&config.pluto_token.to_string())?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: amount,
        })?,
        funds: vec![]
    })];
    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "claim")
        .add_attribute("stage", stage.to_string())
        .add_attribute("address", info.sender.as_str())
        .add_attribute("amount", amount)
        ;

    Ok(res)
}

fn bytes_cmp(a: [u8; 32], b: [u8; 32]) -> std::cmp::Ordering {
    let mut i = 0;
    while i < 32 {
        if a[i] > b[i] {
            return std::cmp::Ordering::Greater;
        } else if a[i] < b[i] {
            return std::cmp::Ordering::Less;
        }

        i += 1;
    }

    return std::cmp::Ordering::Equal;
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::MerkleRoot { stage } => to_binary(&query_merkle_root(deps, stage)?),
        QueryMsg::LatestStage {} => to_binary(&query_latest_stage(deps)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&query_is_claimed(deps, stage, address)?)
        }
    }
}

pub fn query_config(
    deps: Deps,
) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_validate(&state.owner.to_string())?,
        pluto_token: deps.api.addr_validate(&state.pluto_token.to_string())?,
    };

    Ok(resp)
}

pub fn query_merkle_root(
    deps: Deps,
    stage: u8,
) -> StdResult<MerkleRootResponse> {
    let merkle_root = read_merkle_root(deps.storage, stage)?;
    let resp = MerkleRootResponse {
        stage: stage,
        merkle_root: merkle_root,
    };

    Ok(resp)
}

pub fn query_latest_stage(
    deps: Deps,
) -> StdResult<LatestStageResponse> {
    let latest_stage = read_latest_stage(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

pub fn query_is_claimed(
    deps: Deps,
    stage: u8,
    address: HumanAddr,
) -> StdResult<IsClaimedResponse> {
    let user_raw = deps.api.addr_validate(&address)?;
    let resp = IsClaimedResponse {
        is_claimed: read_claimed(deps.storage, &user_raw, stage)?,
    };

    Ok(resp)
}

pub fn migrate(
    _deps: Deps,
    _env: Env,
    _msg: MigrateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}
