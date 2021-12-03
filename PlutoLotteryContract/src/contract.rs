use std::ops::Add;

use crate::msg::{HandleMsg, InstantiateMsg, LoterriesResponse, NewLotteryMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, BankMsg, Binary,  Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, to_binary, CosmosMsg, Decimal
};
use crate::taxation::deduct_tax;
use crate::error::ContractError;
use crate::state::{State, Lottery, LotteryType, Player, PlayerRange,
    config, lottery_store, lottery_read, read_lotteries};

// const PERCENT_TO_DISTRIBUTOR: u64 = 15;
// const PERCENT_TO_JACKPOT: u64 = 5;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        owner: _info.sender,
        winner_handler: _deps.api.addr_validate(&_msg.winner_handler)?,
        lottery_count: 0,
        distribution_addr: _deps.api.addr_validate(&_msg.distribution_addr)?,
        percent_to_distributor: _msg.percent_to_distributor,
        percent_to_jackpot: _msg.percent_to_jackpot
    };
    config(_deps.storage).save(&state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: HandleMsg) -> Result<Response, ContractError> {
    match msg {
        HandleMsg::UpdateConfig {
            owner,
            distribution_addr,
            percent_to_distributor,
            percent_to_jackpot
        } => update_config(
            deps,
            info,
            owner,
            distribution_addr,
            percent_to_distributor,
            percent_to_jackpot
        ),
        HandleMsg::AddPlayerFastDraw { lottery_id } => add_player_fastdraw(deps, env, info, lottery_id),
        HandleMsg::AddPlayerDailyDraw { lottery_id, number_of_tickets } => add_player_dailydraw(deps, env, info, lottery_id, number_of_tickets),
        HandleMsg::RemoveLottery { lottery_id } => remove_lottery(deps, env, info, lottery_id),
        HandleMsg::AddLottery { lottery } =>
            add_new_lottery(deps, env, info, lottery),
        HandleMsg::RewardWinner { lottery_id } => reward_winner(deps, env, info, lottery_id),
    }
}

pub fn add_new_lottery(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lottery: NewLotteryMsg
) -> Result<Response, ContractError> {
    let mut state = config(_deps.storage).load()?;
    // only owner can add new lottery
    if _info.sender != state.owner  {
        return Err(ContractError::Unauthorized {});
    }

    if lottery.lottery_type != LotteryType::Jackpot {
        if lottery.jackpot_lottery_id.is_none() {
            return Err(ContractError::JackpotLotteryNotFound {});
        }
        let jackpot_lottery: Lottery = lottery_read(_deps.storage).load(&lottery.jackpot_lottery_id.unwrap().to_be_bytes())?;

        if jackpot_lottery.lottery_type != LotteryType::Jackpot {
            return Err(ContractError::JackpotCrateFailed {});
        }

        // denoms should be equal
        if jackpot_lottery.denom != lottery.denom {
            return Err(ContractError::WrongDenom {});
        }

    } else {
        if lottery.entry_fee.is_some() || lottery.jackpot_lottery_id.is_some() {
            return Err(ContractError::JackpotCrateFailed {});
        }
    }

    let new_lottery_id = state.lottery_count + 1;
    // Increase lotteries count
    state.lottery_count += 1;

    let new_lottery = Lottery {
        id: new_lottery_id,
        entry_fee: lottery.entry_fee,
        denom: lottery.denom,
        lottery_type: lottery.lottery_type,
        staked_tokens: Uint128::zero(),
        latest_winner: None,
        jackpot_lottery_id : lottery.jackpot_lottery_id,
        players: vec![],
        round_id: 0
    };

    lottery_store(_deps.storage).save(&new_lottery_id.to_be_bytes(), &new_lottery)?;
    config(_deps.storage).save(&state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "add_new_lottery"),
        ("new_lottery_id", &new_lottery_id.to_string()),
    ]))
}

pub fn remove_lottery(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lottery_id: u64
) -> Result<Response, ContractError> {
    let state = config(_deps.storage).load()?;
    // only owner can do this
    if _info.sender != state.owner  {
        return Err(ContractError::Unauthorized {});
    }

    lottery_store(_deps.storage).remove(&lottery_id.to_be_bytes());

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_lottery"),
        ("lottery_id", &lottery_id.to_string()),
    ]))
}

pub fn add_player_fastdraw(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lottery_id: u64
) -> Result<Response, ContractError> {
    let mut lottery: Lottery = lottery_store(_deps.storage).load(&lottery_id.to_be_bytes())?;

    if lottery.lottery_type != LotteryType::Fast {
        return Err(ContractError::Unauthorized {})
    }

    // fast draw for 5 players
    if lottery.players.len() >= 5 {
        return Err(ContractError::ContractFull {})
    }

    if lottery.players.iter().any(|x|  x.addr == _info.sender.clone()) {
        return Err(ContractError::AlreadyExist {});
    }

    // Check funds
    let funds_amount = match _info.funds.len() {
        0 =>  Err(ContractError::WrongFunds {}),
        1 => {
            if _info.funds[0].denom == lottery.denom &&
                _info.funds[0].amount == lottery.entry_fee.unwrap()
            {
                Ok(_info.funds[0].amount.clone())
            } else {
                Err(ContractError::WrongFunds {})
            }
        }
        _ => Err(ContractError::WrongFunds {}),
    }?;

    // add player to lottery
    lottery.players.push(Player { addr: _info.sender.clone(), tickets_count: 1, ticket_num: lottery.players.len() as i32 + 1});

    // increase lottery coins store
    let staked_tokens = lottery.staked_tokens + funds_amount;
    lottery.staked_tokens = staked_tokens;

    lottery_store(_deps.storage).save(&lottery_id.to_be_bytes(), &lottery)?;

    let res = Response::new()
        .add_attribute("players", lottery.players.len().to_string());

    return Ok(res)
}

pub fn add_player_dailydraw(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lottery_id: u64,
    number_of_tickets: u64
) -> Result<Response, ContractError> {
    let mut lottery: Lottery = lottery_store(_deps.storage).load(&lottery_id.to_be_bytes())?;

    if lottery.lottery_type != LotteryType::Daily {
        return Err(ContractError::Unauthorized {})
    }

    // Check funds
    let funds_amount = match _info.funds.len() {
        0 =>  Err(ContractError::WrongFunds {}),
        1 => {
            if _info.funds[0].denom == lottery.denom &&
                _info.funds[0].amount == ( lottery.entry_fee.unwrap().checked_mul( Uint128::from(number_of_tickets) ).unwrap() )
            {
                Ok(_info.funds[0].amount.clone())
            } else {
                Err(ContractError::WrongFunds {})
            }
        }
        _ => Err(ContractError::WrongFunds {}),
    }?;

    // if player exist increase his tickets
    match lottery.players.iter_mut().find(|x| x.addr == _info.sender.clone()) {
        Some(player) => { player.tickets_count = player.tickets_count.add( number_of_tickets as i32)},
        None => { lottery.players.push(Player { addr: _info.sender.clone(), tickets_count: number_of_tickets as i32, ticket_num: lottery.players.len() as i32 + 1}) }
    }
    // increase lottery coins store
    let staked_tokens = lottery.staked_tokens + funds_amount;
    lottery.staked_tokens = staked_tokens;

    lottery_store(_deps.storage).save(&lottery_id.to_be_bytes(), &lottery)?;

    let res = Response::new()
        .add_attribute("players", lottery.players.len().to_string());

    return Ok(res)
}

pub fn reward_winner(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lottery_id: u64
) -> Result<Response, ContractError> {
    let state = config(_deps.storage).load()?;

    if _info.sender != state.owner && _info.sender != state.winner_handler{
        return Err(ContractError::Unauthorized {});
    }

    let mut lottery: Lottery = lottery_store(_deps.storage).load(&lottery_id.to_be_bytes())?;
    let tickets_sum: i32 = lottery.players.iter().map(|x| x.tickets_count).sum();

    if lottery.lottery_type == LotteryType::Fast && tickets_sum < 5 {
        return Err(ContractError::NotEnoughPlayers {})
    } else if tickets_sum < 1 {
        return Err(ContractError::NotEnoughPlayers {})
    }

    let mut final_amount = lottery.staked_tokens.clone();
    let primary_amount = final_amount.clone();

    // section: define winner
        let random_num = (_env.block.height + _env.block.time.seconds()) % tickets_sum as u64;

        // need to define numeric ranges depends on how many tickets player has
        // ranges looks like 0..2, 2..5, 5..14 etc.
        let mut prev_num = 0;
        let ranges: Vec<PlayerRange> = lottery.players
            .iter()
            .map(|x| {
                let last_num = x.tickets_count + prev_num;
                let range: PlayerRange =  PlayerRange {
                    addr: x.addr.clone(),
                    tickets_range: prev_num..last_num
                };
                prev_num = last_num;
                range
            })
            .collect();

        // 0..1 <- it's a range
        // >= < <- random num should be between
        let winner = ranges
            .iter()
            .find(|&x| (random_num as i32) >= x.tickets_range.start  &&
                                    (random_num as i32) < x.tickets_range.end  )
            .unwrap();

        let winner_addr = winner.addr.clone();
    // end section

    // to distributor
    let amount_to_distributor = primary_amount * state.percent_to_distributor;
    //let amount_to_distributor = get_percent_value(primary_amount, PERCENT_TO_DISTRIBUTOR);

    let distributor_msg = BankMsg::Send {
        to_address: state.distribution_addr.to_string(),
        amount: vec![deduct_tax(
            &_deps.querier,
            Coin {
                denom: lottery.denom.clone(),
                amount: amount_to_distributor,
            },
        )?],
    };
    final_amount = final_amount.checked_sub(amount_to_distributor).unwrap();

    // find jackpot lottery and
    // adding tokens and player to it
    if lottery.lottery_type != LotteryType::Jackpot {
        let mut major_jackpot: Lottery = lottery_store(_deps.storage).load(&lottery.jackpot_lottery_id.unwrap().to_be_bytes())?;
        let amount_to_jackpot = primary_amount * state.percent_to_jackpot;
        //let amount_to_jackpot = get_percent_value(primary_amount, PERCENT_TO_JACKPOT);

        major_jackpot.staked_tokens += amount_to_jackpot;
        // jackpot tokens placed in this contract
        final_amount = final_amount.checked_sub(amount_to_jackpot).unwrap();

        let count_of_players_tickets = &lottery.players.iter()
            .find(|&player| player.addr == winner_addr)
            .unwrap()
            .tickets_count;
        // For 1 ust/luna user get 1 ticket for weekly jackpot, so for winning lottery 5ust user will get 5 tickets
        let jackpot_tickets_count = count_of_players_tickets.clone() as u128 * (lottery.entry_fee.unwrap().u128() / 1000000 as u128);
        // winner already exist or not
        match major_jackpot.players.iter_mut().find(|x| x.addr == winner_addr.clone()) {
            Some(player) => { player.tickets_count = player.tickets_count.add( jackpot_tickets_count as i32)},
            None => { major_jackpot.players.push(Player { addr: winner_addr.clone(), tickets_count: jackpot_tickets_count as i32, ticket_num: major_jackpot.players.len() as i32 + 1}) }
        }

        lottery_store(_deps.storage).save(&lottery.jackpot_lottery_id.unwrap().to_be_bytes(), &major_jackpot)?;
    }

    let prev_round_id = lottery.round_id;
    lottery.players = vec![];
    lottery.staked_tokens = Uint128::zero();
    lottery.round_id = prev_round_id + 1;
    lottery.latest_winner = Some(winner_addr.clone().to_string());
    // to winner
    let msg = BankMsg::Send {
        to_address: winner_addr.to_string(),
        amount: vec![deduct_tax(
            &_deps.querier,
            Coin {
                denom: lottery.denom.clone(),
                amount: final_amount,
            },
        )?],
    };

    lottery_store(_deps.storage).save(&lottery_id.to_be_bytes(), &lottery)?;

    let msgs = vec![
            CosmosMsg::Bank(msg),
            CosmosMsg::Bank(distributor_msg),
        ];
    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("tickets_sum", tickets_sum.to_string())
        .add_attribute("num", random_num.clone().to_string())
        .add_attribute("winner", winner_addr.to_string())
        .add_attribute("primary_amount", primary_amount.clone())
        .add_attribute("final_amount", final_amount.clone())
        .add_attribute("round_id",  prev_round_id.to_string());

    return Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    _deps: DepsMut,
    _info: MessageInfo,
    owner: Option<String>,
    distribution_addr: Option<String>,
    percent_to_distributor: Option<Decimal>,
    percent_to_jackpot: Option<Decimal>,
) -> Result<Response, ContractError> {
    let api = _deps.api;
    config(_deps.storage).update(|mut config| {
        if config.owner != _info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = api.addr_validate(&owner)?;
        }

        if let Some(distribution_addr) = distribution_addr {
            config.distribution_addr =  api.addr_validate(&distribution_addr)?;
        }

        if let Some(percent_to_distributor) = percent_to_distributor {
            config.percent_to_distributor =  percent_to_distributor;
        }

        if let Some(percent_to_jackpot) = percent_to_jackpot {
            config.percent_to_jackpot =  percent_to_jackpot;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}


// percentage of value
// fn get_percent_value(value: Uint128, percent: u64) -> Uint128 {
//     let percent_value = value.clone().checked_mul(Uint128::new(percent as u128)).unwrap();
//     let amount = percent_value.checked_div(Uint128::new(100)).unwrap();

//     return amount
// }

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Lotteries {} => to_binary(&query_lotteries(_deps)?)
    }
}

fn query_lotteries(_deps: Deps) -> StdResult<LoterriesResponse> {
    let lotteries = read_lotteries(_deps.storage).unwrap();

    Ok(LoterriesResponse {
        lotteries: lotteries
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, coins, CosmosMsg};
    use crate::msg::{InstantiateMsg};

    #[test]
    fn proper_init() {
        let mut deps = mock_dependencies(&[]);
        let init_msg = InstantiateMsg {
            distribution_addr: "distribution_addr".to_string(),
            winner_handler: "ad".to_string(),
            percent_to_distributor: "0.15",
            percent_to_jackpot: "0.05"
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("addr1", &[]), init_msg).unwrap();
    }

    #[test]
    fn find_daily_winner() {
        let mut deps = mock_dependencies(&[]);
        let init_msg = InstantiateMsg {
            distribution_addr: "distribution_addr".to_string(),
            winner_handler: "ad".to_string(),
            percent_to_distributor: "0.15",
            percent_to_jackpot: "0.05"
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("addr1", &[]), init_msg).unwrap();

        let jackpot = NewLotteryMsg {
            denom: "uusd".to_string(),
            lottery_type: LotteryType::Jackpot,
            jackpot_lottery_id: None,
            entry_fee: None
        };
        let result = add_new_lottery(deps.as_mut(), mock_env(), mock_info("addr1", &[]), jackpot).unwrap();

        let daily = NewLotteryMsg {
            denom: "uusd".to_string(),
            lottery_type: LotteryType::Daily,
            jackpot_lottery_id: Some(1),
            entry_fee: Some(Uint128::new(1000000))
        };
        let result = add_new_lottery(deps.as_mut(), mock_env(), mock_info("addr1", &[]), daily).unwrap();

        add_player_dailydraw(deps.as_mut(), mock_env(), mock_info("addr2", &[Coin {
            denom: "uusd".to_string(),
            amount:Uint128::new(1000000),
        }]),2, 1).unwrap();
        add_player_dailydraw(deps.as_mut(), mock_env(), mock_info("addr3", &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(3000000),
        }]),2, 3).unwrap();

        let lotteries = query_lotteries(deps.as_ref()).unwrap();
        println!("{:?}", lotteries);

        let reward_winner = reward_winner(deps.as_mut(), mock_env(), mock_info("addr1", &[]), 2).unwrap();
        println!("{:?}", reward_winner);
    }

}
