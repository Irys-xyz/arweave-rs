# arweave-rs

SDK for iteracting with Arweave using Rust **(NOT official)**

# Introduction to Use

First you should to check your network whether is ok or not?

```rust
// This link is the one you will visit and test afterward
let arweave_url = Url::parse("https://arweave.net").unwrap();
let network_client = arweave_rs::network::NetworkInfoClient::new(arweave_url);
let info = network_check.network_info().await;
println!("{:?}", info.unwrap());
```

If your network is work, then it output as this:

```
NetworkInfo { network: "arweave.testnet", version: 5, release: 45, height: 0, ...}
```

## A simple way to upload data

We suppose your network is work, and your may want to upload data into arweave. You just need to follow these steps:

1. create `Arewave`

   ```rust
   let arweave_url = Url::parse("https://arweave.net")?;
   let arweave_connect = Arweave::from_keypair_path(
       PathBuf::from("your_jwt_path.json"),
       arweave_url.clone()
   )?;
   ```

   

2. prepare the target, data, and fee.

```rust
let target = Base64(vec![]);
let data = vec![1,2,3];
let fee = arweave_connect.get_fee(target.clone(), data.clone()).await?;
```

3. create transaction and sign it.

```rust
let send_transaction = arweave_connect.create_transaction(
    target,
    vec![],
    data,
    0,
    fee,
    true
).await?;
        
let signed_transaction = arweave_connect.sign_transaction(send_transaction)?;
```

4. In the end, you need to post transaction and receive the id.

```rust
let result = arweave_connect.post_transaction(&signed_transaction).await?;
```



