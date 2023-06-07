use subxt::{OnlineClient, PolkadotConfig};

use crate::query::maybe_leases;

pub enum Chain {
    DOT,
    KSM,
    ROC,
}

pub type Api = OnlineClient<PolkadotConfig>;

// Returns if the passed para_id is applicable for a permanent slot in Rococo
pub async fn needs_perm_slot(para_id: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let polkadot_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io:443").await?;
    let kusama_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://kusama-rpc.polkadot.io:443").await?;

    let lease_polkadot = maybe_leases(polkadot_api, Chain::DOT, para_id).await;

    let lease_kusama = maybe_leases(kusama_api, Chain::KSM, para_id).await;

    if lease_kusama.unwrap() || lease_polkadot.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Returns if the passed para_id already has a slot in Rococo
pub async fn has_slot_in_rococo(para_id: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let rococo_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://rococo-rpc.polkadot.io:443").await?;

    let lease_rococo = maybe_leases(rococo_api, Chain::ROC, para_id).await;

    if lease_rococo.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}
