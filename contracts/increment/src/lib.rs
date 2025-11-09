#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct Contract;

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl Contract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}

mod test;

// Optional: deployment / run logs (kept as comments for reference)
/*
ℹ️ Using wasm hash 7382fc901f0fa857b820e1b0e307469b81ef70b21dfb127ff18b97a01c8eed2c
ℹ️ Simulating deploy transaction…
ℹ️ Transaction hash is 667da7c7a6ce59336d71b2953120364c99f0ba0a90e4e57770966b8333618bce
🔗 https://stellar.expert/explorer/testnet/tx/667da7c7a6ce59336d71b2953120364c99f0ba0a90e4e57770966b8333618bce
ℹ️ Signing transaction: 667da7c7a6ce59336d71b2953120364c99f0ba0a90e4e57770966b8333618bce
🌎 Submitting deploy transaction…
🔗 https://stellar.expert/explorer/testnet/contract/CC3XSYJBRUXDH4VFL6QUV72OSNRJ6CVQLR47DFMSXORJNIEYNOS4J32O
✅ Deployed!
⚠️ Overwriting existing alias "increment" that currently links to contract ID: CAX6YD3AMD7MIBUFPYEUZBNXSL7HNHXISRUYBAXMLUUJ6T3OVXVD6CNR
CC3XSYJBRUXDH4VFL6QUV72OSNRJ6CVQLR47DFMSXORJNIEYNOS4J32O
*/
