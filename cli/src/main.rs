use clap::Parser;
use dotenv::dotenv;
use para_onboarding::{
    calls::{
        create_batch_all_call, create_force_register_call, create_force_transfer_call,
        create_scheduled_assign_slots_call, create_scheduled_remove_lock_call, create_sudo_call,
        sign_and_send_proxy_call, Call,
    },
    chain_connector::{kusama_connection, polkadot_connection, rococo_connection},
    utils::{
        calculate_sovereign_account, get_file_content, has_slot_in_rococo, is_registered,
        needs_perm_slot, parse_validation_code,
    },
};
use sp_core::sr25519::Pair;
use subxt::utils::AccountId32;

#[derive(Parser, Debug)]
#[command(about = "CLI tool to onboard parachains.")]
struct Cli {
    /// Parachain ID
    para_id: String,
    /// Parachain manager account
    manager_account: String,
    /// Path to a file with a genesis head.
    #[clap(long, short('g'), value_parser)]
    path_genesis_head: String,
    /// Path to the wasm file.
    #[clap(long, short('v'), value_parser)]
    path_validation_code: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Cli::parse();

    let para_id: u32 = args.para_id.parse::<u32>().unwrap();
    let manager_account: AccountId32 = args.manager_account.parse().unwrap();

    let rococo_api = rococo_connection().await;

    // Don't do anything if the ParaID already has a slot assigned in Rococo
    let has_slot: bool = has_slot_in_rococo(rococo_api.clone(), para_id.clone())
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

    let is_perm_slot: bool = needs_perm_slot(polkadot_api, kusama_api, para_id.clone())
        .await
        .unwrap_or(false);
    // prints just for testing, remove before publishing
    if is_perm_slot {
        println!("ParaId: {} needs a permanent slot", para_id.clone());
    } else {
        println!("ParaId: {} needs a temporary slot", para_id.clone());
    }

    // Initialise an empty call buffer
    let mut call_buffer: Vec<Call> = Vec::<Call>::new();

    // If the ParaID is not registered register it with sudo
    let is_registered = is_registered(rococo_api.clone(), para_id.clone()).await;
    if !is_registered.unwrap() {
        println!("ParaId is not registered, registering now.");
        // Add call to send funds to `manager_account`
        call_buffer.push(create_force_transfer_call(manager_account.clone()).unwrap());

        let genesis_head = get_file_content(args.path_genesis_head).await;
        let validation_code = get_file_content(args.path_validation_code).await;

        let genesis_bytes = genesis_head.as_bytes().to_vec();
        println!("got Genesis file");
        let validation_code_bytes = parse_validation_code(validation_code);
        println!("got WASM file");

        call_buffer.push(
            create_force_register_call(
                para_id.clone(),
                manager_account,
                genesis_bytes,
                validation_code_bytes,
            )
            .unwrap(),
        );
        println!("force_register call preapred \n {:?}", call_buffer[call_buffer.len() - 1]);
    }

    // Add call to send funds to paras sovereign account
    call_buffer.push(
        create_force_transfer_call(calculate_sovereign_account::<Pair>(para_id.clone()).unwrap())
            .unwrap(),
    );
    println!("send funds to soveregins account call prepared\n {:?}", call_buffer[call_buffer.len() - 1]);

    // Add call to schedule assigning a slot to the given para
    call_buffer.push(create_scheduled_assign_slots_call(para_id.clone(), is_perm_slot).unwrap());

    println!("create_scheduled_assign_slots_call prepared\n {:?}", call_buffer[call_buffer.len() - 1]);
    // Add call to schedule removing the manager lock from the given para
    call_buffer.push(create_scheduled_remove_lock_call(para_id).unwrap());

    println!("create_scheduled_remove_lock_call prepared\n {:?}", call_buffer[call_buffer.len() - 1]);
    // Get the batched call based on the calls present in buffer
    let batch_call = create_batch_all_call(call_buffer).unwrap();

    println!("batch calls");
    // Create a SUDO call
    let sudo_call = create_sudo_call(batch_call).unwrap();
    println!("crearte sudo call\n {:?}", &sudo_call);

    // Sign and send batch_call to the network
    if let Err(subxt::Error::Runtime(dispatch_err)) =
        sign_and_send_proxy_call(rococo_api, sudo_call).await
    {
        eprintln!("Could not dispatch the call: {}", dispatch_err);
    }
    Ok(())
}
