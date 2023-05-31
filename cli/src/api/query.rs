use subxt::{OnlineClient, PolkadotConfig};
use std::error::Error;

#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_metadata.scale")]
pub mod polkadot {}

#[subxt::subxt(runtime_metadata_path = "metadata/kusama_metadata.scale")]
pub mod kusama {}

use polkadot::runtime_types::polkadot_parachain::primitives::Id;
use kusama::runtime_types::polkadot_parachain::primitives::Id as KusmamaId;

pub async fn exists_in_polkadot(para_id: u32) -> Result<bool, Box<dyn Error>> {
        let query_param: Id = Id(para_id);
        // Create a new API client, configured to talk to Polkadot nodes.
        let polkadot_api = OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io:443").await?;

        let storage_query = polkadot::storage().registrar().paras(query_param);

        let result = polkadot_api
            .storage()
            .at_latest()
            .await?
            .fetch(&storage_query)
            .await?;
        
        match result {
            Some(_) => return Ok(true),
            None => return Ok(false),
        }  
}

pub async fn exists_in_kusama(para_id: u32) -> Result<bool, Box<dyn Error>>  {
    let query_param: KusmamaId = KusmamaId(para_id);
    // Create a new API client, configured to talk to Polkadot nodes.
    let kusama_api = OnlineClient::<PolkadotConfig>::from_url("wss://kusama-rpc.polkadot.io:443").await?;

    let storage_query = kusama::storage().registrar().paras(query_param);

    let result = kusama_api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;
    
    match result {
        Some(_) => return Ok(true),
        None => return Ok(false),
    }
}
