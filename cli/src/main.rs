use clap::Parser;
use dotenv::dotenv;
use para_onboarding::{
    chain_connector::{kusama_connection, polkadot_connection, rococo_connection},
    utils::{has_slot_in_rococo, is_registered, needs_perm_slot, calculate_sovereign_account, parse_validation_code},
    calls::{Call, create_batch_all_call, create_force_transfer_call,
        create_force_register_call, create_scheduled_assign_slots_call,
        create_scheduled_remove_lock_call, create_sudo_call, sign_and_send_proxy_call
    },
};
use std::{fs, path::PathBuf};
use sp_core::sr25519::Pair;
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

    // Don't do anything if the ParaID already has a slot assigned in Rococo
    let has_slot: bool = has_slot_in_rococo(rococo_api.clone(), args.para_id.clone())
        .await
        .unwrap_or(false);
    if has_slot {
        println!(
            "Error: ParaId: {} already has a slot in Rococo",
            args.para_id.clone()
        );
        return Ok(());
    }

    // Query Polkadot and Kusma to see if the ParaID needs a permanent/tmp slot
    let polkadot_api = polkadot_connection().await;
    let kusama_api = kusama_connection().await;

    let is_perm_slot: bool = needs_perm_slot(polkadot_api, kusama_api, args.para_id.clone())
        .await
        .unwrap_or(false);
    // prints just for testing, remove before publishing
    if is_perm_slot {
        println!("ParaId: {} needs a permanent slot", args.para_id.clone());
    } else {
        println!("ParaId: {} needs a temporary slot", args.para_id.clone());
    }
    
    // Initialise an empty call buffer
    let mut call_buffer: Vec<Call> = Vec::<Call>::new();

    // If the ParaID is not registered (Parachain or Parathread) register it with sudo
    let is_registered = is_registered(rococo_api.clone(), args.para_id.clone()).await;
    if !is_registered.unwrap() {
        
        // Add call to send funds to `manager_account`
        call_buffer.push(create_force_transfer_call(args.manager_account.clone()).unwrap());
        
        // Add call to register the para with the given args
        let validation_code = fs::read_to_string(
            args.path_validation_code
        ).expect("Should have been able to read the validation code file");
        let genesis_head = fs::read(
            args.path_genesis_head
        ).expect("Should have been able to read the genesis file");

        call_buffer.push(create_force_register_call(
            args.para_id.clone(),
            args.manager_account,
            genesis_head,
            parse_validation_code(validation_code),
        ).unwrap());
    }

    // Add call to send funds to paras sovereign account
    call_buffer.push(
        create_force_transfer_call(calculate_sovereign_account::<Pair>(args.para_id.clone()).unwrap()).unwrap()
    );

    // Add call to schedule assigning a slot to the given para
    call_buffer.push(create_scheduled_assign_slots_call(args.para_id.clone(), is_perm_slot).unwrap());

    // Add call to schedule removing the manager lock from the given para
    call_buffer.push(create_scheduled_remove_lock_call(args.para_id).unwrap());

    // Get the batched call based on the calls present in buffer
    let batch_call = create_batch_all_call(call_buffer).unwrap();

    // Create a SUDO call
    let sudo_call = create_sudo_call(batch_call).unwrap();

    // Sign and send batch_call to the network
    sign_and_send_proxy_call(rococo_api, sudo_call).await

}
