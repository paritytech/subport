use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "CLI tool to onboard parachains.")]
struct Cli {
    /// Parachain ID
    para_id: u32,
    /// Manager Address
    account_address: String,
}

fn main() {
    let args = Cli::parse();
    println!("Parachain ID: {}", args.para_id);
    println!("Manager Address: {}", args.account_address);
}
