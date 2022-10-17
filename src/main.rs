use std::fs;
use std::{path::PathBuf, str::FromStr};

use arweave_rs::crypto::base64::Base64;
use arweave_rs::Arweave;
use url::Url;

#[tokio::main]
async fn main() {
    /* let target = Base64::from_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap();
    let path = PathBuf::from_str(".wallet.json").unwrap();
    let arweave =
        Arweave::from_keypair_path(path, Url::from_str("https://arweave.net").unwrap()).unwrap();

    let fee = arweave.get_fee(target).await.unwrap();

    let tx = arweave
        .create_transaction(
            Base64::from_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap(),
            vec![],
            vec![],
            100000,
            fee,
            false,
        )
        .await
        .unwrap();

    let sig_tx = arweave.sign_transaction(tx).unwrap();
    //let ok = arweave.verify_transaction(&sig_tx);
    //dbg!(ok);

    let (id, reward) = arweave.post_transaction(&sig_tx).await.unwrap();

    println!("id: {:?} | reward: {:?}", id.to_string(), reward);

    let (status, tx_status) = arweave
        .get_tx_status(Base64::from_str(&id).unwrap())
        .await
        .unwrap();
    dbg!(status, json!(tx_status));
    */

    let path = PathBuf::from_str(".wallet.json").unwrap();
    let file_path = PathBuf::from_str("data").unwrap();
    let target = Base64::empty();

    let arweave =
        Arweave::from_keypair_path(path, Url::from_str("https://arweave.net").unwrap()).unwrap();
    let data = fs::read(file_path.clone()).expect("Could not read file");

    let fee = arweave.get_fee(target, data).await.unwrap();

    let res = arweave.upload_file_from_path(file_path, vec![], fee).await;

    println!("{:?}", res);
}
