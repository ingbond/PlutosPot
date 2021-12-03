import { logger } from './../services/logger';
import express from "express";
import { getLotteries } from "../db/lottery-db";
import { getWinners } from "../db/winner-history-db";
import { toPascalCase } from "../lib/caseStyles";
import { lcd } from "../lib/terra/lcd";
import { CharityEndowements } from "../models/contract/charity-endowements";
import { LotteriesResponse, Player } from "../models/contract/loterries-response.model";
import { LotteryData, LotteryFullRecord } from "../models/view/lottery-full-record.model";
import { checkSum } from "../services/check-sum";

exports.getLotteriesInfo = async (req:  express.Request, res:  express.Response) => {
    let contractLotteriesResult =  await lcd!.wasm
    .contractQuery(process.env.LOTTERY_CONTRACT!, {
        lotteries: {},
    })
    .then(async (data: any) => {            
        let response: LotteriesResponse = toPascalCase(data);
        return response;
    })
    .catch(er => {
        logger.error(er);
        return { Lotteries: [] } as LotteriesResponse;        
    });

        
    let dbLotteries = await getLotteries();
    let winners = await getWinners();
    let fullRecords: LotteryFullRecord[] = [];

    contractLotteriesResult.Lotteries.forEach(lottery => {
        let dbLottery = dbLotteries?.find(x => x.Id == lottery.Id);
        let record: LotteryFullRecord = {
            LotteryData: {...lottery, ...dbLottery} as LotteryData,
            Winners: winners?.filter(x => x.LotteryId == lottery.Id) ?? [],
        };

        fullRecords.push(record);
    })

    res.status(200).json(fullRecords)
}; 

exports.getCharityEndowements  = async (req:  express.Request, res:  express.Response) => {
    await lcd!.wasm
    .contractQuery(process.env.DISTRIBUTOR_CONTRACT!, {
        state: {},
    })
    .then(async (data: any) => {
        let response: CharityEndowements = toPascalCase(data);
        res.status(200).json(response);
    })
    .catch(er => {
        res.status(500).json(er);
        logger.error(er);
    });    
}


exports.checkSum = async (req:  express.Request, res:  express.Response) => {
    try {
        await checkSum();
        res.status(200).send({message: 'Sum is correct'})
    } catch (e) {
        console.log("er" + e.message);
        res.status(500).send({message: e.message})
    }    
}; 

exports.getLotteryAddr = async (req:  express.Request, res:  express.Response) => {
    let addr = process.env.LOTTERY_CONTRACT;
    res.status(200).json({Addr: addr})
}; 

