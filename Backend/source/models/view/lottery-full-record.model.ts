import { PlayerDB } from './../db/player.model';

import { Winner } from '../db/winner.model';
import { LotteryType, Player } from './../contract/loterries-response.model';
import { CharityEndowements } from '../contract/charity-endowements';

export interface LotteryFullRecord {
    LotteryData: LotteryData,
    Winners: Winner[]
}

export interface LotteryData {
    Id: number,
    EntryFee?: number,
    Denom: String,
    LotteryType: LotteryType,
    StakedTokens: number,
    LatestWinner?: String,
    JackpotLotteryId?: number,
    LastWinnerDefinitionDate?: Date,
    HoursBetweenWinnerRewarding?: number,
    Players: Player[]
}
