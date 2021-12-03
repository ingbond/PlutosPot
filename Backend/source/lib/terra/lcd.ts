import { LCDClient, TxInfo, Wallet, Msg, Coins, Coin, LocalTerra } from '@terra-money/terra.js'
import { delay } from 'bluebird'

export let lcd: LCDClient;


export async function initLCD(URL: string, chainID: string): Promise<LCDClient> {
  if(!URL || !chainID) 
    lcd = new LocalTerra();
  else 
    lcd = new LCDClient({ URL, chainID })

  return lcd
}

export async function checkTx(txHash: string, timeout = 60000): Promise<TxInfo | undefined> {
  const startedAt = Date.now()

  while (Date.now() - startedAt < timeout) {
    const txInfo = await lcd!.tx.txInfo(txHash).catch(() => undefined)

    if (txInfo) {
      return txInfo  
    }

    await delay(1000)
  }
}

export async function transaction(
  wallet: Wallet,
  msgs: Msg[],
  fee: undefined,
  accountNumber = undefined,
  sequence = undefined,
  timeout = 60000
): Promise<TxInfo | undefined> {
  return wallet
    .createAndSignTx({ msgs, account_number: accountNumber, sequence, fee })
    .then((signed) => lcd!.tx.broadcast(signed))
    .then(async (broadcastResult: any) => {
      if (broadcastResult['code']) {
        throw new Error(broadcastResult.raw_log)
      }
      return checkTx(broadcastResult.txhash, timeout)
    })
}

export async function contractInfo<T>(address: string): Promise<any> {
  if (!address) {
    throw new Error('wrong address')
  }
  return await lcd!.wasm.contractInfo(address)
}

export async function contractQuery(
  address: string,
  query: Record<string, unknown>
): Promise<any> {
  if (!address) {
    throw new Error('wrong address')
  }
  return await lcd!.wasm.contractQuery(address, query) 
}
