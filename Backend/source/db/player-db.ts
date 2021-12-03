import { logger } from '../services/logger';
import { PlayerDB } from './../models/db/player.model';
import { mainpool } from "./connection/pg-connections";

export async function getRecordsOfPlayersByLotteryAndRound(lotteryId: number, roundId: number): Promise<PlayerDB[] | undefined> {
    try {
        const result = await mainpool.query(`select * from public."Players" where "LotteryId" = ${lotteryId} and "RoundId" = ${roundId}`);
        return (result.rows as PlayerDB[]);
    } catch (err) {
        logger.error(`getRecordsOfPlayersByLotteryAndRound SQL error: ${err}`);
    }
}

export async function getPlayerRecordsByAddr(addr: string): Promise<PlayerDB[] | undefined> {
    try {
        const result = await mainpool.query(`select * from public."Players" where "Addr" = '${addr}'`);
        return (result.rows as PlayerDB[]);
    } catch (err) {
        logger.error(`getPlayerRecordsByAddr SQL error: ${err}`);
    }
}

export async function getPlayerRecordsByAddrAndLottery(addr: string, lotteryId: number, roundId: number): Promise<PlayerDB | undefined> {
  try {
      const result = await mainpool.query(`select * from public."Players" where "Addr" = '${addr}' and "LotteryId" = ${lotteryId} and "RoundId" = ${roundId} limit 1`);
      return (result.rows[0] as PlayerDB);
  } catch (err) {
    logger.error(`getPlayerRecordsByAddrAndLottery SQL error: ${err}`);
  }
}

export async function getPlayers(): Promise<PlayerDB[] | undefined> {
    try {
        const result = await mainpool.query(`select * from public."Players" limit 100`);
        return (result.rows as PlayerDB[]);
    } catch (err) {
        logger.error(`getPlayers SQL error: ${err}`);
    }
}

export async function insertPlayer(addr: string, ticketsCount: number, lotteryId: number, roundId: number, ticketNum: number) {
    logger.info(`insertPlayer ${addr} ${ticketsCount} ${lotteryId} ${roundId} ${ticketNum}`);
    try {
        const query = {
            text: 'insert into public."Players" ("Addr", "TicketsCount", "LotteryId", "RoundId", "Date", "TicketNum") VALUES($1, $2, $3, $4, $5, $6)',
            values: [addr, ticketsCount, lotteryId, roundId, new Date().toUTCString(), ticketNum],
          }
        const result = await mainpool.query(query);
        return result.rows;
    } catch (err) {
        logger.error(`insertPlayer SQL error: ${err}`);
    }
}
