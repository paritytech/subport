use clap::Parser;
use para_onboarding::helper::{has_slot_in_rococo, needs_perm_slot};

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

    let has_already_exist: bool = has_slot_in_rococo(args.para_id).await.unwrap_or(false);
    if has_already_exist {
        println!(
            "Error: ParaId: {} already has a slot in Rococo",
            args.para_id
        );
        return Ok(());
    }

    let perm_slot: bool = needs_perm_slot(args.para_id).await.unwrap_or(false);

    if perm_slot {
        println!("ParaId: {} needs a permanent slot", args.para_id);
    } else {
        println!("ParaId: {} needs a temporary slot", args.para_id);
    }

    Ok(())
}
