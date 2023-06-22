use clap::Parser;
use para_onboarding::helper::{has_slot_in_rococo, needs_perm_slot, register, is_registered};
use std::{fs, path::PathBuf};
use subxt::{tx::PairSigner, utils::AccountId32};

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
        register(
            args.para_id,
            args.account_address,
            args.path_validation_code,
            args.path_genesis_head,
        )
        .await?;
    }


    let perm_slot: bool = needs_perm_slot(args.para_id).await.unwrap_or(false);

    if perm_slot {
        println!("ParaId: {} needs a permanent slot", args.para_id);
    } else {
        println!("ParaId: {} needs a temporary slot", args.para_id);
    }

    Ok(())
}
