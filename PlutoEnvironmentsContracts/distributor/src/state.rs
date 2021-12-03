use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read};

static KEY_CONFIG: &[u8] = b"config";
static KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub angel_addr: Addr,
    pub operation_addr: Addr,
    pub collector_addr: Addr,
    pub reward_factor_operation: Decimal, // reward distribution rate to operation_addr
    pub reward_factor_collector: Decimal, // reward distribution rate to collector_addr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateData {
    pub accumulated_angel_amount_ust: Uint128,
    pub accumulated_angel_amount_luna: Uint128,
}


pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

pub fn store_state(storage: &mut dyn Storage, state: &StateData) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<StateData> {
    singleton_read(storage, KEY_STATE).load()
}

