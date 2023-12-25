use std::str::FromStr;

use cosm_utils::{
    chain::{
        coin::{Coin, Denom},
        request::TxOptions,
    },
    config::cfg::ChainConfig,
    modules::bank::{api::BankTxCommit, model::SendRequest},
    signing_key::key::{Key, UserKey},
    tendermint_rpc::HttpClient,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    mnemonic: String,
    derivation_path: String,
    prefix: String,
    chain_id: String,
    denom: String,
    memo: String,
    rpc: String,
    gas_price: f64,
    gas_adjustment: f64,
    times: u64,
}

async fn send_tokens(config: Config) -> anyhow::Result<()> {
    let key = UserKey {
        name: "Pizda".to_string(),
        key: Key::Mnemonic(config.mnemonic.to_string()),
    };

    let address = key
        .public_key(&config.derivation_path)
        .await
        .unwrap()
        .account_id(&config.prefix)
        .unwrap();

    let chain_cfg = ChainConfig {
        denom: config.denom,
        prefix: config.prefix,
        chain_id: config.chain_id,
        derivation_path: config.derivation_path,
        gas_price: config.gas_price,
        gas_adjustment: config.gas_adjustment,
    };
    let req = SendRequest {
        from: address.clone().into(),
        to: address.into(),
        amounts: vec![Coin {
            denom: Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
            amount: 1u128,
        }],
    };
    let tx_options = TxOptions {
        memo: config.memo,
        ..Default::default()
    };

    let client = HttpClient::new(config.rpc.as_str()).unwrap();

    let res = client
        .bank_send_commit(&chain_cfg, req, &key, &tx_options)
        .await
        .unwrap();

    println!("{}", res.hash);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config_data = String::new();
    tokio::fs::OpenOptions::new()
        .read(true)
        .open("./config.toml")
        .await?
        .read_to_string(&mut config_data)
        .await?;

    let config: Config = toml::from_str(&config_data)?;

    for _ in 0..(config.times) {
        send_tokens(config.clone()).await.unwrap();
    }

    Ok(())
}
