use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Addr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub angel_addr: String,
    pub operation_addr: String,
    pub reward_factor_operation: Decimal,
    pub collector_addr: Addr,
    pub reward_factor_collector: Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update config interface
    /// to enable reward_factor update
    UpdateConfig {
        owner: Option<String>,
        collector_addr:  Option<String>,
        angel_addr:  Option<String>,
        operation_addr:  Option<String>,
        reward_factor_operation:  Option<Decimal>
    },
    /// execute Distribute message
    Distribute { denom: String },
    Withdraw { denom: String, addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {}
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub angel_addr: Addr,
    pub operation_addr: Addr,
    pub reward_factor_operation: Decimal,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
