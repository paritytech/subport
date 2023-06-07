use crate::helper::{Chain, Api};

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_metadata.scale")]
pub mod polkadot {}

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_metadata.scale")]
pub mod kusama {}

#[subxt::subxt(runtime_metadata_path = "metadata/rococo_metadata.scale")]
pub mod rococo {}

use polkadot::runtime_types::polkadot_parachain::primitives::Id;
use kusama::runtime_types::polkadot_parachain::primitives::Id as KusamaId;
use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;


// Checks if paraId holds any leases on the specified chain
//
pub async fn maybe_leases(
    api: Api,
    chain: Chain,
    para_id: u32
) -> Result<bool, Box<dyn std::error::Error>> {
    
    let query = match chain {
        Chain::DOT => polkadot::storage().slots().leases(Id(para_id)),
        Chain::KSM => kusama::storage().slots().leases(KusamaId(para_id)),
        Chain::ROC => rococo::storage().slots().leases(RococoId(para_id)),
    };
    
    match api
        .storage()
        .at_latest()
        .await?
        .fetch(&query)
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}