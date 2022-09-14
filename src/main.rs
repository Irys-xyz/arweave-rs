use arweave_rs::{crypto::base64::Base64, Arweave};

#[tokio::main]
async fn main() {
    let arweave = Arweave::default();
    let tx = arweave
        .create_w2w_transaction(
            Base64::from_utf8_str("PAgdonEn9f5xd-UbYdCX40Sj28eltQVnxz6bbUijeVY").unwrap(),
            vec![],
            10000000000,
            677952,
            true,
        )
        .await
        .unwrap();

    let sig_tx = arweave.sign_transaction(tx).unwrap();
    println!("{:?}", sig_tx);
    let res = arweave.post_transaction(&sig_tx).await;

    println!("{:?}", res);
}
