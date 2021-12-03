use crate::contract::{execute, instantiate, query_config, reply};
use crate::mock_querier::mock_dependencies;
use crate::collector::{ConfigResponse, ExecuteMsg, InstantiateMsg};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    to_binary, Coin, ContractResult, CosmosMsg, Decimal, Reply, ReplyOn, StdError, SubMsg,
    SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::ExecuteMsg as TerraswapExecuteMsg;

