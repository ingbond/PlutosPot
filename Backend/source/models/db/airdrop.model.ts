export interface AirdropModel {
    Id: number | null;
    Stage: number;
    Address: String;
    Staked: number;
    Rate: number;
    Amount: number;
    Total: number;
    Proof: string;
    MerkleRoot: string;
    IsClaimed: boolean;
}
