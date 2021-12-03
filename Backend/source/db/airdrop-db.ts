import { AirdropModel } from './../models/db/airdrop.model';
import { logger } from "../services/logger";
import { mainpool } from "./connection/pg-connections";

export async function getAirdrop(stage: number, contractAddress: string) {
  try {
    const result = await mainpool.query(`select * from public."Airdrops" where "Stage" = ${stage} AND "Address" = '${contractAddress}'`);
      return (result.rows[0] as AirdropModel);
  } catch (err) {
      logger.error(`getAirdrop SQL error: ${err}`);
  }
}

export async function getAirdrops(contractAddress: string) {
  try {
    const result = await mainpool.query(`select * from public."Airdrops" where "Address" = '${contractAddress}'`);
      return (result.rows as AirdropModel[]);
  } catch (err) {
      logger.error(`getAirdrops SQL error: ${err}`);
  }
}

export async function saveAirdrop(airdrop: AirdropModel) {
  logger.info(`saveAirdrop : ${JSON.stringify(airdrop)}`);
  try {
      const query = {
          text: `insert into public."Airdrops" 
          ("Stage", "Address", "Rate", "Amount", "Total", "Proof", "MerkleRoot", "Staked", "IsClaimed") 
          VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
          values: [airdrop.Stage, airdrop.Address, airdrop.Rate, airdrop.Amount, airdrop.Total, airdrop.Proof, airdrop.MerkleRoot, airdrop.Staked, airdrop.IsClaimed]
        }
      const result = await mainpool.query(query);
      return result.rows;
  } catch (err) {
      logger.error(`saveAirdrop SQL error: ${err}`);
  }
}

export async function setAirdropClaimed(id: number) {
  logger.info(`setAirdropClaimed: ${id}`);
  try {
      const query = {
          text: `update public."Airdrops" set "IsClaimed" = 'true' where "Id" = $1`,
          values: [id],
        }
      const result = await mainpool.query(query);
      return result.rows;
  } catch (err) {
      logger.error(`setAirdropClaimed SQL error: ${err}`);
  }
}

