import { RewardWinner } from "../models/db/reward-winner.model";
import { Winner as RewardHistoryRecord } from "../models/db/winner.model";
import { logger } from "../services/logger";
import { mainpool } from "./connection/pg-connections";

export async function getWinnersAfterDate(date: Date): Promise<RewardHistoryRecord[] | undefined> {
    try {
       const result = await mainpool.query(`select * from public."RewardsHistory" where "Date" > '${date.toUTCString()}'`);
       return (result.rows as RewardHistoryRecord[]);
    } catch (err) {
        logger.error(`getWinnersAfterDate SQL error: ${err}`);
    }
  }

export async function getLatestLotteryWinner(lotteryId: number): Promise<RewardHistoryRecord | undefined> {
    try {
       const result = await mainpool.query(`select * from public."RewardsHistory" where "LotteryId" = ${lotteryId} order by "Date" asc limit 1 `);
       return (result.rows[0] as RewardHistoryRecord);
    } catch (err) {
        logger.error(`getLatestLotteryWinner SQL error: ${err}`);
    }
  }

export async function getWinnerRecorsByAddr(addr: string): Promise<RewardHistoryRecord[] | undefined> {
    try {
       const result = await mainpool.query(`select * FROM public."RewardsHistory" where "WinnerAddr" = '${addr}'`);
       return (result.rows as RewardHistoryRecord[]);
    } catch (err) {
        logger.error(`getWinnerByAddr SQL error: ${err}`);
    }
  }

export async function getWinners(): Promise<RewardHistoryRecord[] | undefined> {
     try {
        const result = await mainpool.query(`select * FROM public."RewardsHistory" limit 100`);
        return (result.rows as RewardHistoryRecord[]);
     } catch (err) {
         logger.error(`getWinners SQL error: ${err}`);
     }
   }

export async function insertWinner(object: RewardWinner) {
  logger.info(`save db winner: ${JSON.stringify(object)}`);
    try {
        const query = {
            text: `
            insert into public."RewardsHistory" ("LotteryId", "WinnerAddr", "Date", "Amount", "AmountBeforeTax", "RoundId") 
                values ($1, $2, $3, $4, $5, $6)`,
            values: [object.LotteryId, object.WinnerAddr,object.Date.toUTCString(), object.Amount, object.AmountBeforeTax, object.RoundId],
          }
        const result = await mainpool.query(query);
        return result.rows;
    } catch (err) {
        logger.error(`insertWinner SQL error: ${err}`);
    }
  }