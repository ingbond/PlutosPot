
import { Lottery } from "../models/db/lottery-db.model";
import { logger } from "../services/logger";
import { mainpool } from "./connection/pg-connections";

export async function getLotteries(): Promise<Lottery[] | undefined> {
     try {
         const result = await mainpool.query(`select * FROM public."Lotteries"`);
         return (result.rows as Lottery[]);
     } catch (err) {
         logger.error(`getLotteries SQL error: ${err}`);
     }
   }

export async function getLotteryById(id: number): Promise<Lottery | undefined> {
  try {
      const result = await mainpool.query(`select * from public."Lotteries" where "Id" = ${id}`);
      return (result.rows[0] as Lottery);
  } catch (err) {
      logger.error(`getLotteryById SQL error: ${err}`);
  }
}

export async function insertLottery(id: number, mins: number) {
    logger.info(`insert Lottery : ${id}`);
    try {
        const query = {
            text: 'insert into public."Lotteries" ("Id", "LastWinnerDefinitionDate", "MinsBetweenWinnerRewarding") VALUES($1, $2, $3)',
            values: [id, new Date().toUTCString(), mins],
          }
        const result = await mainpool.query(query);
        return result.rows;
    } catch (err) {
        logger.error(`insertLottery SQL error: ${err}`);
    }
  }

  export async function updateLastRewardDate(id: number, date: Date) {
    logger.info(`update Last Reward Date : ${id}, ${date}`);
    try {
        const query = {
            text: 'update public."Lotteries" set "LastWinnerDefinitionDate" = $1 where "Id" = $2',
            values: [new Date().toUTCString(), id],
          }
        const result = await mainpool.query(query);
        return result.rows;
    } catch (err) {
        logger.error(`updateLastRewardDate SQL error: ${err}`);
    }
  }
  
  