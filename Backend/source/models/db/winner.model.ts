export interface Winner {
    Id: number,
    LotteryId: number,
    WinnerAddr: string,
    Date: Date,
    Amount: number,
    AmountBeforeTax: number,
    RoundId: number,
}
