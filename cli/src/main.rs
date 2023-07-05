use clap::Parser;
use std::{path::PathBuf};
use subxt::{utils::AccountId32};
use dotenv::dotenv;
use para_onboarding::helper::{has_slot_in_rococo, needs_perm_slot, register, is_registered, assign_slots, fund_parachain_manager};

#[derive(Parser, Debug)]
#[command(about = "CLI tool to onboard parachains.")]
struct Cli {
    /// Parachain ID
    para_id: u32,
    /// Manager Address
    account_address: AccountId32,
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

    // Don't the anything if the ParaID already has an slot in Rococo
    let has_already_slot: bool = has_slot_in_rococo(args.para_id).await.unwrap_or(false);
    if has_already_slot {
        println!(
            "Error: ParaId: {} already has a slot in Rococo",
            args.para_id
        );
        return Ok(());
    }

    // If the ParaID is not registered (Parachain or Parathread), register it with sudo
    let is_registered = is_registered(args.para_id).await;
    if !is_registered.unwrap() {
        println!("Registering para_id {:?}", args.para_id);
        let registration_result = register(
            args.para_id,
            args.account_address.clone(),
            args.path_genesis_head,
            args.path_validation_code,
        )
        .await;
        match registration_result {
            Ok(_) => println!("Registration succesful"),
            Err(_error) => panic!("Error registrating the parachain"),
        };
    }
    fund_parachain_manager(args.account_address).await;


    let perm_slot: bool = needs_perm_slot(args.para_id).await.unwrap_or(false);
    if perm_slot {
        println!("ParaId: {} needs a permanent slot", args.para_id);
    }
    else {
        println!("ParaId: {} needs a temporary slot", args.para_id);
    }

    let assign_slots_result = assign_slots(args.para_id, perm_slot).await;
    
    match assign_slots_result {
        Ok(_) => println!("Slots scheduled to be assigned"),
        Err(_error) => panic!("Error assigning the slots"),
    };

    Ok(())
}
