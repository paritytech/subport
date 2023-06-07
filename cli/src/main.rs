
use clap::Parser;
use para_onboarding::helper::needs_perm_slot;

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

    let _perm_slot: bool = needs_perm_slot(args.para_id).await.unwrap_or(false);
    
    Ok(())
}
