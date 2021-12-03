use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{State, Lottery, LotteryType};
use cosmwasm_std::{Addr, Uint128, Decimal};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub distribution_addr: String,
    pub winner_handler: String,
    pub percent_to_distributor: Decimal,
    pub percent_to_jackpot: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewLotteryMsg {
    pub denom: String,
    pub entry_fee: Option<Uint128>,
    pub lottery_type: LotteryType,
    pub jackpot_lottery_id: Option<u64>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<String>,
        distribution_addr: Option<String>,
        percent_to_distributor: Option<Decimal>,
        percent_to_jackpot: Option<Decimal>,
    },
    AddPlayerFastDraw { lottery_id: u64 },
    AddPlayerDailyDraw { lottery_id: u64, number_of_tickets: u64 },
    RemoveLottery { lottery_id: u64 },
    AddLottery {
        lottery: NewLotteryMsg
    },
    RewardWinner { lottery_id: u64 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Lotteries {}
}

// We define a custom struct for each query response
pub type ConfigResponse = State;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ActivePlayersResponse {
    /// list all registered ids
    pub active_players: Vec<Addr>,
    pub round_id: Uint128
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct LoterriesResponse {
    pub lotteries: Vec<Lottery>,
}
