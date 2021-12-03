import { MnemonicKey, Wallet } from "@terra-money/terra.js"
import { lcd } from "./lcd";

export let main_wallet: Wallet;

// lcd must be init
export function initWallet() {
    let key = new MnemonicKey({mnemonic: process.env.MNEMONIC});
    let wallet = new Wallet(lcd!, key);
    main_wallet = wallet;
}