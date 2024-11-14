### 初始化
```
let ethereum_client = EthereumClient::new(
        &etherum.endpoint,
        &etherum.chain_name,
        etherum.chain_id,
        etherum.start_height,
        etherum.contracts.clone(),
    )
    .await;
```
