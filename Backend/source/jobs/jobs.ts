import { initAirdropQueue } from "./airdrop-job";
import { initCheckSumQueue } from "./check-sum-job";
import { initFindWinnerQueue } from "./find-winner-job";

export async function initWorkers() {    
      await initFindWinnerQueue({
        port: +process.env.REDIS_PORT!,
        host: process.env.REDIS_HOST!
      });

      await initCheckSumQueue({
        port: +process.env.REDIS_PORT!,
        host: process.env.REDIS_HOST!
      });

      await initAirdropQueue({
        port: +process.env.REDIS_PORT!,
        host: process.env.REDIS_HOST!
      });
      
}
