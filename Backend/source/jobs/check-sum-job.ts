import { QueueScheduler, Queue, Worker } from "bullmq";
import { checkSum } from "../services/check-sum";

export async function initCheckSumQueue(connection: { host: string, port: number})
 {  
    const sh = new QueueScheduler('CheckSum', { connection: {
        host: connection.host,
        port: connection.port
      }});
    const queue = new Queue('CheckSum', { connection: {
        host: connection.host,
        port: connection.port
      }});
    await clearQueue(queue);
    await queue.add(
        'Sync', 
        {},
        {
            repeat: {
                every: +(process.env.CHECK_SUM_FREQUENCY_MINS ?? 60) * 60000
            },
            jobId: 'sum'
        }        
    );

    new Worker('CheckSum', async job => {
        if (job.name === 'Sync') {
            await checkSum();
        }
      }, { connection: {
        host: connection.host,
        port: connection.port
      }});
 }

 export async function clearQueue(queue: Queue)
 {   
    await queue.remove("sum");
    await queue.removeRepeatable("CheckSum", {}); 
    await queue.obliterate({ force: true });    
 }