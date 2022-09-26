use std::{path::PathBuf, str::FromStr};

use arweave_rs::{ Arweave };
use arweave_rs::crypto::base64::Base64;
use serde_json::json;
use url::Url;

#[tokio::main]
async fn main() {
    let path = PathBuf::from_str(".wallet.json").unwrap();
    let arweave = 
        Arweave::from_keypair_path(path, Url::from_str("https://arweave.net").unwrap())
        .unwrap();
    
    let price_terms = arweave
        .get_price_terms(1.0)
        .await
        .unwrap();
    
    let tx = arweave.create_w2w_transaction(
        Base64::from_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap(),
        vec![],
        100000,
        (1005093808, 0),
    false)
        .await
        .unwrap();
    dbg!(json!(&tx));
        
    let sig_tx = arweave
        .sign_transaction(tx)
        .unwrap();

    
    let res = arweave
        .post_transaction(&sig_tx)
        .await;

    dbg!(res);
}
