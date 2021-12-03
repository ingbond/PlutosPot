import { MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { QueueScheduler, Queue, Worker } from "bullmq";
import { INITIAL_AIRDROP_AMOUNT, INITIAL_AIRDROP_BLOCK_HEIGHT, LUNA_STAKER_AIRDROP_AMOUNT, SNAPSHOT_BLOCK_PERIOD, SNAPSHOT_BLOCK_START, SNAPSHOT_LAST_STAGE } from "../common/initial-values";
import { getAirdrop, saveAirdrop } from "../db/airdrop-db";
import { contractQuery, lcd } from "../lib/terra/lcd";
import { main_wallet } from "../lib/terra/wallet";
import Airdrop from "../services/airdrop";
import { logger } from "../services/logger";
import Snapshot from "../services/snapshot";


export async function initAirdropQueue(connection: { host: string, port: number})
{  
   const sh = new QueueScheduler('Airdrop', { connection: {
       host: connection.host,
       port: connection.port
     }});
   const queue = new Queue('Airdrop', { connection: {
       host: connection.host,
       port: connection.port
     }});

    await clearQueue(queue);

    await queue.add(
        'Sync', 
        {},
        {
            repeat: {
                every:  +(process.env.AIRDROP_FREQUENCY_MINS ?? 1440) * 60000               
            },
            jobId: 'airdropId'
        }        
    );
        
    new Worker('Airdrop', async job => {
        console.log('Airdrop');
        if (job.name === 'Sync') {
            const latestBlock = await lcd!.tendermint.blockInfo()            
            if (!latestBlock) {
                return
            }            
            const response: any = await contractQuery(process.env.AIRDROP!, { latest_stage: {} });      
            const latestStage = response.latest_stage;

            if (!latestStage) {
                // take initial airdrop snapshot
                await takeSnapshot(main_wallet, 1, INITIAL_AIRDROP_BLOCK_HEIGHT, Number(INITIAL_AIRDROP_AMOUNT))

                return
            }
            
            const latestBlockHeight = +latestBlock.block.header.height;
            const nextStage = latestStage + 1;
            const nextStageHeight = SNAPSHOT_BLOCK_START + ((nextStage - 2) * SNAPSHOT_BLOCK_PERIOD);

            if (nextStage <= SNAPSHOT_LAST_STAGE && latestBlockHeight - 10 >= nextStageHeight) {
              await takeSnapshot(main_wallet, nextStage, nextStageHeight, Number(LUNA_STAKER_AIRDROP_AMOUNT))
            }
        }
      }, { connection: {
        host: connection.host,
        port: connection.port
      }});
 }


async function takeSnapshot(
    wallet: Wallet, stage: number, height: number, airdropAmount: number
  ): Promise<void> {
    logger.info(`takeSnapshot ${new Date()}`);
    // take snapshot
    const snapshot = new Snapshot(lcd?.config.URL!);
    const snapshotHeight = height - (height % 10000);
    const delegators = await snapshot.takeSnapshot(snapshotHeight);

    const delegatorAddresses = Object.keys(delegators);
    const delegatorTokens = Object.values(delegators);

    if (delegatorAddresses.length < 1) {
      logger.error('take snapshot failed. target delegators is none.');
      throw new Error('take snapshot failed. target delegators is none.')
    }

    // calculate total staked luna amount
    const total: number = Number(delegatorTokens.reduce((s, x) => s + x))
    if (!total) {
      logger.error('calculate total failed');
      throw new Error('calculate total failed')
    }   
  
    // calculate airdrop amount per account
    const accounts: any[] = []
    try {  
      delegatorAddresses.map((delegator: string) => {
        // кол-во стака у делегатора
        const staked = Number(delegators[delegator]);
        // стак деленное на общеее количество всех стакнутых токенов
        const rate = staked / total;
        // доля делегатору от токенов, выделенных на аирдроп
        const amount = parseFloat((airdropAmount * rate).toFixed(0));
        if (amount  > 0) {
          accounts.push({ address: delegator, amount, staked: staked, rate })
        }
      })
    } catch(error) {
      throw new Error(error)
    }
    const airdrop = new Airdrop(accounts)
    const merkleRoot = airdrop.getMerkleRoot();

    // save airdrop information to db
    accounts.forEach(async account => {
        const { address, staked, rate, amount } = account
        const proof = airdrop.getMerkleProof({ address, amount })
        // already exist
        if (await getAirdrop(stage, address)) {
            return
        }
    
        await saveAirdrop({
            Id: null,
            Stage: stage,
            Address: address,
            Staked: staked,
            Rate: rate,
            Amount: amount,
            Total: total,
            Proof: JSON.stringify(proof),
            MerkleRoot: merkleRoot,
            IsClaimed: false
        })
    })

    const execute = new MsgExecuteContract(
        wallet.key.accAddress, // sender
        process.env.AIRDROP!, // contract account address
        { register_merkle_root: { "merkle_root": merkleRoot } }, // handle msg
      );

    // register merkle root
    const executeTx = await wallet.createAndSignTx({
        msgs: [execute],
        // fee: new StdFee(1000,"1000000" + 'uusd')
      }).then(async (tx: any) => {
        var result = await lcd!.tx.broadcast(tx);
        logger.info(`register merkle root: ${JSON.stringify(result)}`);
      }).catch(er => {
        logger.error('er data ', er.response.data.error);
      }); 
  }


 export async function clearQueue(queue: Queue)
 {   
    await queue.remove("airdropId");
    await queue.removeRepeatable("Airdrop", {}); 
    await queue.obliterate({ force: true });    
 }