use clap::Parser;
use dotenv::dotenv;
use para_onboarding::{
    chain_connector::{kusama_connection, polkadot_connection, rococo_connection},
    helper::{
        assign_slots, fund_parachain_manager, fund_sovereign_account, has_slot_in_rococo,
        is_registered, needs_perm_slot, register, remove_parachain_lock,
    },
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

    // If the ParaID is not registered (Parachain or Parathread), register it with sudo
    let is_registered = is_registered(rococo_api.clone(), args.para_id).await;
    if !is_registered.unwrap() {
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
    let lock_removed = remove_parachain_lock(rococo_api.clone(), args.para_id).await;
    match lock_removed {
        Ok(_) => println!("Lock removed for the parachain"),
        Err(_error) => panic!("Error removing the lock for the parachain"),
    };
    let parachain_funded = fund_parachain_manager(rococo_api.clone(), args.manager_account).await;
    match parachain_funded {
        Ok(_) => println!("Funds sent to the manager account"),
        Err(_error) => panic!("Error sending funds the manager account"),
    };

    let sovereign_account_funded = fund_sovereign_account(rococo_api.clone(), args.para_id).await;
    match sovereign_account_funded {
        Ok(_) => println!("Funds sent to the sovereign account account"),
        Err(_error) => panic!("Error sending funds the sovereign account"),
    };

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

    let assign_slots_result = assign_slots(rococo_api.clone(), args.para_id, perm_slot).await;

    match assign_slots_result {
        Ok(_) => println!("Slots scheduled to be assigned"),
        Err(_error) => panic!("Error assigning the slots"),
    };

    Ok(())
}
