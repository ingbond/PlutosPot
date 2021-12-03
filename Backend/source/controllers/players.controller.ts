import { PlayerData } from './../models/view/player-data.model';
import express from "express";
import { getPlayerRecordsByAddr, getPlayers } from "../db/player-db";
import { getWinnerRecorsByAddr, getWinners } from "../db/winner-history-db";
import { getAirdrops, setAirdropClaimed } from '../db/airdrop-db';
import { contractQuery } from '../lib/terra/lcd';

exports.getPlayerData = async (req:  express.Request, res:  express.Response) => {
    let addr = req.params.addr;

    if(!addr) {
        res.status(400);
        return;
    }

    let data = await getPlayerRecordsByAddr(addr);
    let playerData: PlayerData[] = [];

    if (data) {
        let winners = await getWinnerRecorsByAddr(addr);
        data.forEach(player => {
            let winRecord = winners?.find(x => x.WinnerAddr == player.Addr && x.RoundId == player.RoundId && x.LotteryId == player.LotteryId);
            let resultPlayer: PlayerData = {
                Addr: player.Addr,
                TicketNum: player.TicketNum ?? null,
                LotteryId: player.LotteryId,
                RoundId: player.RoundId,
                IsWinner: !!winRecord,
                Amount: winRecord?.Amount ?? null,
                TicketsCount: player.TicketsCount,
                Date: player.Date ?? null,
                RewardDate: winRecord?.Date ?? null
            };

            playerData.push(resultPlayer);
        });
    }

    res.status(200).json(playerData)
}; 


exports.getHistory = async (req:  express.Request, res:  express.Response) => {
    let players = await getPlayers();
    let winners = await getWinners();

    let playersData: PlayerData[] = [];

    if (players) {
        players.forEach(player => {
            let winner = winners?.find(x => x.WinnerAddr == player.Addr && x.RoundId == player.RoundId && x.LotteryId == player.LotteryId);
            let resultPlayer: PlayerData = {
                Addr: player.Addr,
                TicketNum: player.TicketNum ?? null,
                LotteryId: player.LotteryId,
                RoundId: player.RoundId,
                IsWinner: !!winner,
                Amount: winner?.Amount ?? null,
                TicketsCount: player.TicketsCount,
                Date: player.Date ?? null,
                RewardDate: winner?.Date ?? null
            };

            playersData.push(resultPlayer);
        });
    }
    res.status(200).json(playersData)
}; 

exports.getAirdrops = async (req:  express.Request, res:  express.Response) => {
    let addr = req.params.addr;

    if(!addr) {
        res.status(400);
        return;
    }

    let airdrops = await getAirdrops(addr);

    if(airdrops) {
        for(let i = 0; i < airdrops.length; i++) {
            let airdrop = airdrops[i];

            if(!airdrop.IsClaimed) {
                const response: any = await contractQuery(process.env.AIRDROP!, { is_claimed: { "stage": airdrop.Stage, "address": airdrop.Address} });
                console.log(response);
                if (response.is_claimed)  {
                    await setAirdropClaimed(airdrop.Id!);
                    airdrop.IsClaimed = true;
                }
            }
        }
    }

    res.status(200).json(airdrops)
}; 
