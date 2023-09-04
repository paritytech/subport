use crate::utils::Chain;
use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_metadata.scale")]
pub mod polkadot {}

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_metadata.scale")]
pub mod kusama {}

#[subxt::subxt(runtime_metadata_path = "metadata/rococo_metadata.scale")]
pub mod rococo {}
// #[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
// pub mod rococo {}

use kusama::runtime_types::polkadot_parachain::primitives::Id as KusamaId;
use polkadot::runtime_types::polkadot_parachain::primitives::Id;
use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;

//
// Checks if paraId holds any leases on the specified chain
//
pub async fn maybe_leases(
    api: OnlineClient<PolkadotConfig>,
    chain: Chain,
    para_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let query = match chain {
        Chain::DOT => polkadot::storage().slots().leases(Id(para_id)),
        Chain::KSM => kusama::storage().slots().leases(KusamaId(para_id)),
        Chain::ROC => rococo::storage().slots().leases(RococoId(para_id)),
    };

    match api.storage().at_latest().await?.fetch(&query).await? {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}

//
// Check the next free para available in in Rococo
//
pub async fn next_free_para(api: OnlineClient<PolkadotConfig>) -> u32 {
    let query = rococo::storage().registrar().next_free_para_id();
    let id = api
        .storage()
        .at_latest()
        .await
        .expect("Error getting the next free para id")
        .fetch(&query)
        .await
        .unwrap()
        .expect("Error getting the next free para id");
    id.0
}

//
// Checks if paraId is already registered in Rococo
//
pub async fn paras_registered(
    api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let query = rococo::storage().paras().para_lifecycles(RococoId(para_id));
    match api.storage().at_latest().await?.fetch(&query).await? {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
