import { LotteryContract } from './../models/contract/loterries-response.model';
import { getRecordsOfPlayersByLotteryAndRound } from "../db/player-db";
import { getLatestLotteryWinner, getWinnersAfterDate } from "../db/winner-history-db";
import { toPascalCase } from "../lib/caseStyles";
import { lcd } from "../lib/terra/lcd";
import { LotteriesResponse } from "../models/contract/loterries-response.model";
import { logger } from './logger';
import { Extension } from '@terra-money/terra.js';

export const checkSum = async () => {  
  await lcd!.wasm
    .contractQuery(process.env.LOTTERY_CONTRACT!, {
      lotteries: {},
    })
    .then(async (data: any) => {
      let response: LotteriesResponse = toPascalCase(data);
      await checkLotteriesSum(response);      
    })
    .catch(er => {
      logger.error(er);
    });
};

export async function checkLotteriesSum(data: LotteriesResponse) {
  let lotteries = data.Lotteries;
  let jackpotLotteries = data.Lotteries.filter(x => x.LotteryType == "jackpot");
  
  let ustLottery = jackpotLotteries.find(x => x.Denom == "uusd")!;
  let lunaLottery = jackpotLotteries.find(x => x.Denom == "uluna")!;

  await processLottery(lotteries, ustLottery);
  await processLottery(lotteries, lunaLottery);
}

async function processLottery(lotteries: LotteryContract[], jackpotLottery: LotteryContract) {
  logger.info(`Check sum start: jackpotLotteryId: ${jackpotLottery.Id}; staked tokens: ${jackpotLottery.StakedTokens}`);
  // need on first time
  let weekAgo = new Date();
  weekAgo.setDate(weekAgo.getDate() - 7);

  let jackpotLatestWinnerDate = (await getLatestLotteryWinner(jackpotLottery!.Id))?.Date ?? weekAgo;
  // get all winners in db after last jackpot
  let winnersAfterDate = (await getWinnersAfterDate(jackpotLatestWinnerDate))?.filter(x => x.LotteryId != jackpotLottery!.Id);

  if (!winnersAfterDate) {
    return;
  }

  let sumWinnersAmount = 0;
  
  for(let i = 0; i < winnersAfterDate.length; i++) {
    let winner = winnersAfterDate[i];
    let lottery = lotteries.find(x => x.Id == winner.LotteryId);

    if (lottery?.Denom != jackpotLottery.Denom) {
      continue;
    }
    /* The goal is to check that amount that winner receive is correct 
      So getting all players in lottery and 
        sum tickets count * entry fee = amount that winner should receive
    */
    let players = await getRecordsOfPlayersByLotteryAndRound(winner.LotteryId, winner.RoundId);

    if (players) {
      let targetAmount = players.reduce((pv, cv) => pv + cv.TicketsCount * lottery!.EntryFee, 0);
      if(targetAmount != winner.AmountBeforeTax) {
        let err = `WRONG Amount of players sum. winner id:${winner.Id}: amount: ${targetAmount} != ${winner.AmountBeforeTax}`;
        logger.error(err);
        throw new Error(err);
      }
    }
    sumWinnersAmount += winner.AmountBeforeTax;
  }
  
  /* 5% of amount, that winners receive during week, should be stored in contract */
  if(sumWinnersAmount * 0.05 != jackpotLottery?.StakedTokens){    
    let err = `WRONG SUM of staked tokens. jackpot id:${jackpotLottery.Id} sum: ${sumWinnersAmount * 0.05} != ${jackpotLottery?.StakedTokens}`;
    logger.error(err);
    throw new Error(err);
  }  

  logger.info(`Check sum complete: ${new Date()}. Jackpot id: ${jackpotLottery.Id}; db sum: ${sumWinnersAmount} (${sumWinnersAmount * 0.05}) = contract sum: ${jackpotLottery?.StakedTokens}`);
}