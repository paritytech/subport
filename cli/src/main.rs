
use clap::Parser;

mod api;
use api::query::{exists_in_polkadot, exists_in_kusama};


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

    let is_in_polkadot = exists_in_polkadot(args.para_id).await;
    let is_in_kusama = exists_in_kusama(args.para_id).await;
    let is_exists = match is_in_polkadot {
        Ok(is_exists) => 
            match is_in_kusama {
                Ok(is_exists_kusama) => is_exists || is_exists_kusama,
                Err(v) => Err(format!("Error querying the chain: {}", v))?
            },
        Err(v) => Err(format!("Error querying the chain: {}", v))?
    };
    if is_exists {
        // Chain exists on Polkadot/Kusama -> long term
        println!("Parachain exists");
    }
    else {
        // Chain does not exist on Polkadot/Kusama -> short term
        println!("This parachain does not exist");
    }
    
    Ok(())
}
