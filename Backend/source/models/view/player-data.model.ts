export interface PlayerData {
    Addr: string,
    LotteryId: number,
    RoundId: number,
    TicketNum: number | null,
    IsWinner: boolean,
    Amount: number | null,
    TicketsCount: number,
    Date: Date | null,
    RewardDate: Date | null
}
