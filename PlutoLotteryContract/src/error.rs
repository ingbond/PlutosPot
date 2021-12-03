use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not enough players")]
    NotEnoughPlayers {},

    #[error("Too many players")]
    ContractFull {},

    #[error("Player already exist")]
    AlreadyExist {},

    #[error("No funds or wrong value")]
    WrongFunds {},

    #[error("Denom is wrong")]
    WrongDenom {},

    #[error("Must provide jackpot lottery id")]
    JackpotLotteryNotFound {},

    #[error("Jackpot can't contains jacpot id or entree fee")]
    JackpotCrateFailed {},
}
