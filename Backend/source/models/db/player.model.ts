export interface PlayerDB {
    Id: number,
    Addr: string,
    LotteryId: number,
    RoundId: number,
    TicketNum?: number,
    TicketsCount: number,
    Date?: Date
}
