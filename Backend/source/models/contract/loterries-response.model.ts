export interface LotteriesResponse {
    Lotteries: LotteryContract[];
}

export interface LotteryContract {
    Id: number,
    EntryFee: number,
    Denom: String,
    LotteryType: LotteryType,
    Players: Player[],
    StakedTokens: number,
    LatestWinner: String,
    JackpotLotteryId: number,
    RoundId: number,
}

export type LotteryType = "fast" | "daily" | "jackpot";

export interface Player {
    Addr: string;
    TicketsCount: number;
    TicketNum: number;
}