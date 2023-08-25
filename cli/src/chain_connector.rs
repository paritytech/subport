use subxt::{OnlineClient, PolkadotConfig};

pub async fn rococo_connection() -> OnlineClient<PolkadotConfig> {
    let uri = std::env::var("ROCOCO_URI").unwrap_or("ws://127.0.0.1:9944".to_string());
    let rococo_api = OnlineClient::<PolkadotConfig>::from_url(uri)
        .await
        .expect("Connection to Rococo failed");
    rococo_api
}

pub async fn polkadot_connection() -> OnlineClient<PolkadotConfig> {
    let polkadot_api = OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io:443")
        .await
        .expect("Connection to Rococo failed");
    polkadot_api
}

pub async fn kusama_connection() -> OnlineClient<PolkadotConfig> {
    let kusama_api = OnlineClient::<PolkadotConfig>::from_url("wss://kusama-rpc.polkadot.io:443")
        .await
        .expect("Connection to Rococo failed");
    kusama_api
}
