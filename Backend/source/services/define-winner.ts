import { FASTDRAW_DEFENITION_MINS, DAILY_DEFENITION_MINS, JACKPOT_DEFENITION_MINS } from './../common/initial-values';
import { LotteryType } from "../models/contract/loterries-response.model";
import {
  FASTDRAW_PLAYERS_COUNT,
} from "../common/initial-values";
import {
  BlockTxBroadcastResult,
  LocalTerra,
  MsgExecuteContract, StdTx,
} from "@terra-money/terra.js";
import {
  getLotteryById,
  insertLottery,
  updateLastRewardDate,
} from "../db/lottery-db";
import { LotteriesResponse } from "../models/contract/loterries-response.model";
import { insertWinner } from "../db/winner-history-db";
import { lcd } from "../lib/terra/lcd";
import { main_wallet } from "../lib/terra/wallet";
import { toPascalCase } from "../lib/caseStyles";
import { getPlayerRecordsByAddrAndLottery, insertPlayer } from '../db/player-db';
import { checkLotteriesSum } from './check-sum';
import { logger } from './logger';

type HrsDefinition = Record<LotteryType, number>;
const lottery_rewarding_gap: HrsDefinition = {
  jackpot: JACKPOT_DEFENITION_MINS,
  daily: DAILY_DEFENITION_MINS,
  fast: FASTDRAW_DEFENITION_MINS,
};

export const tryDefineWinners = async () => { 
  await lcd!.wasm
    .contractQuery(process.env.LOTTERY_CONTRACT!, {
      lotteries: {},
    })
    .then(async (data: any) => {
      let response: LotteriesResponse = toPascalCase(data);
      await processLotteries(response); 
    })
    .catch(err => logger.error(`try Define Winners err ${err.message}`));
};

async function processLotteries(data: LotteriesResponse) {
  var lotteries = data.Lotteries;
  logger.info(`process lotteries at ${new Date().toString()}`)

  for (let i = 0; i < lotteries.length; i++) {
    let now = new Date();
    let lottery = lotteries[i];
    let dbLottery = await getLotteryById(lottery.Id);
    // need to have actual lottery in the database
    if (!dbLottery) {
      await insertLottery(
        lottery.Id,
        lottery_rewarding_gap[lottery.LotteryType]
      );

      dbLottery = await getLotteryById(lottery.Id);
    }

    lottery.Players.forEach(async player => {
      let dbPlayer = await getPlayerRecordsByAddrAndLottery(player.Addr, lottery.Id, lottery.RoundId);
      if(!dbPlayer) {
        await insertPlayer(player.Addr, player.TicketsCount, lottery.Id, lottery.RoundId, player.TicketNum);
      }
    });

    // find next date for rewarding winner
    let rewardDate = addHours(
      dbLottery!.LastWinnerDefinitionDate,
      dbLottery!.MinsBetweenWinnerRewarding
    );

    if (now >= rewardDate) {
      switch (lottery.LotteryType) {
        case "jackpot":
        case "daily":
          if (lottery.Players.length >= 1) {
            await handleWinner(lottery.Id);
          } else {
            await updateLastRewardDate(lottery.Id, new Date());
          }
          break;
        case "fast":
          if (lottery.Players.length >= FASTDRAW_PLAYERS_COUNT) {
            await handleWinner(lottery.Id);
          }
          break;
      }
    }
  }
}

async function handleWinner(lotteryId: number) {
  logger.info(`handle winner  id:${lotteryId}  date:${new Date()}`);
  const wallet = main_wallet;
  const execute = new MsgExecuteContract(
    wallet.key.accAddress, // sender
    process.env.LOTTERY_CONTRACT!, // contract account address
    { reward_winner: { lottery_id: lotteryId } } // handle msg
  );
  const executeTx = await wallet
    .createAndSignTx({
      msgs: [execute],
    })
    .then(async (tx) => {
      var result = await lcd!.tx.broadcast(tx);
      await processBroadcastResult(result, lotteryId);
    })
    .catch((er) => {
      logger.error(`er on handle Winner ${er.response.data.error}`);
    });

  return executeTx;
}

async function processBroadcastResult(result: BlockTxBroadcastResult, lotteryId: number) {
  let resultObj: { key: string; value: string }[] = JSON.parse(
    result.raw_log
  )[0].events.find((x: any) => x.type == "from_contract").attributes;
  // update latest reward date in the database
  await updateLastRewardDate(lotteryId, new Date());
  // store winner in the db
  await insertWinner({
    Id: null, // will set by db
    LotteryId: lotteryId,
    WinnerAddr:
      resultObj.find((x) => x.key == "winner")?.value ??
      "Not Returned From The Contract",
    Date: new Date(),
    Amount: +(resultObj.find((x) => x.key == "final_amount")?.value ?? 0),
    AmountBeforeTax: +(
      resultObj.find((x) => x.key == "primary_amount")?.value ?? 0
    ),
    RoundId: +(
      resultObj.find((x) => x.key == "round_id")?.value ?? 0
    )
  });
}

function addHours(date: Date, mins: number): Date {
  var copiedDate = new Date(date.getTime());
  copiedDate.setTime(copiedDate.getTime() + mins * 60 * 1000);
  return copiedDate;
}
