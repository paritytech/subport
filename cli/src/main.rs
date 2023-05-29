#[subxt::subxt(runtime_metadata_path = "metadata/polkadot_metadata.scale")]
pub mod polkadot {}

use clap::Parser;
use subxt::{OnlineClient, PolkadotConfig};


use polkadot::runtime_types::polkadot_parachain::primitives::{Id};

#[derive(Parser, Debug)]
#[command(about = "CLI tool to onboard parachains.")]
struct Cli {
    /// Parachain ID
    para_id: u32,
    /// Manager Address
    account_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("Parachain ID: {}", args.para_id);
    println!("Manager Address: {}", args.account_address);
    // Create a new API client, configured to talk to Polkadot nodes.
    let api = OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io:443").await?;
    // let para_id = args.para_id.into();
    let tx_params: Id = Id(args.para_id);
    println!("{:?}",tx_params);
    let storage_query = polkadot::storage().registrar().paras(tx_params);

    let result = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;

    println!("{:?}",result);
    Ok(())
}
