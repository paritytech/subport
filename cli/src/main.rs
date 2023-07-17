use clap::Parser;
use dotenv::dotenv;
use para_onboarding::{
    chain_connector::{kusama_connection, polkadot_connection, rococo_connection},
    helper::{batch_calls, has_slot_in_rococo, is_registered, needs_perm_slot, register, fund_account},
};
use std::path::PathBuf;
use subxt::utils::AccountId32;

#[derive(Parser, Debug)]
#[command(about = "CLI tool to onboard parachains.")]
struct Cli {
    /// Parachain ID
    para_id: u32,
    /// Parachain manager account
    manager_account: AccountId32,
    /// Path to a file with a genesis head.
    #[clap(long, short('g'), value_parser)]
    path_genesis_head: PathBuf,
    /// Path to the wasm file.
    #[clap(long, short('v'), value_parser)]
    path_validation_code: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Cli::parse();

    let rococo_api = rococo_connection().await;

    // Don't do anything if the ParaID already has an slot in Rococo
    let has_slot: bool = has_slot_in_rococo(rococo_api.clone(), args.para_id)
        .await
        .unwrap_or(false);
    if has_slot {
        println!(
            "Error: ParaId: {} already has a slot in Rococo",
            args.para_id
        );
        return Ok(());
    }
    // Query Polkadot and Kusma to see if the ParaID needs a permanent/tmp slot
    let polkadot_api = polkadot_connection().await;
    let kusama_api = kusama_connection().await;

    let perm_slot: bool = needs_perm_slot(polkadot_api, kusama_api, args.para_id)
        .await
        .unwrap_or(false);
    if perm_slot {
        println!("ParaId: {} needs a permanent slot", args.para_id);
    } else {
        println!("ParaId: {} needs a temporary slot", args.para_id);
    }

    // If the ParaID is not registered (Parachain or Parathread), register it with sudo
    let is_registered = is_registered(rococo_api.clone(), args.para_id).await;
    if !is_registered.unwrap() {
        
        // Send some funds to `manager_account`
        fund_account(args.manager_account);

        println!("Registering para_id {:?}", args.para_id);
        let registration_result = register(
            rococo_api.clone(),
            args.para_id,
            args.manager_account.clone(),
            args.path_genesis_head,
            args.path_validation_code,
        )
        .await;
        match registration_result {
            Ok(_) => println!("Registration succesful"),
            Err(_error) => panic!("Error registrating the parachain"),
        };
    }

    // Rest of the calls bached: remove parachain lock, fund parachain manager and sovereign account and schedule assign slots
    let calls_batched = batch_calls(
        rococo_api.clone(),
        args.para_id,
        args.manager_account,
        perm_slot,
    )
    .await;
    match calls_batched {
        Ok(_) => println!("A batch of calls has been sent succesfully"),
        Err(_error) => panic!("Error batching the calls"),
    };

    Ok(())
}
