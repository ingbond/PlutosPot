use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Uint128};
use std::ops::Range;

pub static CONFIG_KEY: &[u8] = b"config";
static PREFIX_LOT: &[u8] = b"lottery";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub distribution_addr: Addr,
    pub owner: Addr,
    pub winner_handler: Addr,
    pub lottery_count: u64,
    pub percent_to_distributor: Decimal,
    pub percent_to_jackpot: Decimal,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Lottery {
    pub id: u64,
    pub entry_fee: Option<Uint128>,
    pub denom: String,
    pub lottery_type: LotteryType,
    pub staked_tokens: Uint128,
    pub latest_winner: Option<String>,
    pub jackpot_lottery_id: Option<u64>,
    pub players: Vec<Player>,
    pub round_id: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Player {
    pub addr: Addr,
    pub tickets_count: i32,
    pub ticket_num: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PlayerRange {
    pub addr: Addr,
    pub tickets_range: Range<i32>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LotteryType {
    Fast,
    Daily,
    Jackpot,
}

pub fn lottery_store(storage: &mut dyn Storage) -> Bucket<Lottery> {
    bucket(storage, PREFIX_LOT)
}

pub fn lottery_read(storage: &dyn Storage) -> ReadonlyBucket<Lottery> {
    bucket_read(storage, PREFIX_LOT)
}

pub fn read_lotteries<'a>(storage: &'a dyn Storage) -> StdResult<Vec<Lottery>> {
    let lotteries: ReadonlyBucket<'a, Lottery> = ReadonlyBucket::new(storage, PREFIX_LOT);
    lotteries
        .range(None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}
