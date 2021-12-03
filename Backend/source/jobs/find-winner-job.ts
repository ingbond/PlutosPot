import { QueueScheduler, Queue, Worker, QueueOptions } from "bullmq";
import { tryDefineWinners } from "../services/define-winner";

export async function initFindWinnerQueue(connection: { host: string, port: number})
 {  
    const sh = new QueueScheduler('FindWinners', { connection: {
        host: connection.host,
        port: connection.port
      }});
    const queue = new Queue('FindWinners', { connection: {
        host: connection.host,
        port: connection.port
      }});
    await clearQueue(queue);
    await queue.add(
        'Sync', 
        {},
        {
            repeat: {
                every: +(process.env.FIND_WINNER_FREQUENCY_MINS ?? 60) * 60000
            },
            jobId: 'winner'
        }        
    );

    new Worker('FindWinners', async job => {
        if (job.name === 'Sync') {
            await tryDefineWinners();
        }
      }, { connection: {
        host: connection.host,
        port: connection.port
      }});
 }

 export async function clearQueue(queue: Queue)
 {   
    await queue.remove("winner");
    await queue.removeRepeatable("FindWinners", {}); 
    await queue.obliterate({ force: true });    
 }