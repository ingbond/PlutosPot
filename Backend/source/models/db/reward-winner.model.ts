export interface RewardWinner {
    Id: number | null;
    LotteryId: number;
    WinnerAddr: string;
    Date: Date;
    Amount: number;
    AmountBeforeTax: number;
    RoundId: number;
}
