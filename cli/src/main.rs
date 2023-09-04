use clap::Parser;
use dotenv::dotenv;
use para_onboarding::{
    calls::{
        create_batch_all_call, create_force_register_call, create_force_transfer_call,
        create_scheduled_assign_slots_call, create_scheduled_remove_lock_call, create_sudo_call,
        reserve, sign_and_send_proxy_call, Call,
    },
    chain_connector::{kusama_connection, polkadot_connection, rococo_connection},
    utils::{
        calculate_sovereign_account, get_file_content, get_next_free_para, has_slot_in_rococo,
        is_registered, needs_perm_slot, parse_validation_code,
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

    // If needs a permanent slot, check if the paraId has been reserved and if not, reserve it in Rococo
    //
    // If paraId < nextParaId needs means is reserved,
    // If paraId = nextParaId needs to be reserved
    // If paraId > nextParaId throws an Error, because teh para_id indicated should be the nextParaId
    if !is_perm_slot {
        println!("ParaId: {} needs a temporary slot", para_id.clone());
        let next_para_id = get_next_free_para(rococo_api.clone()).await?;
        if next_para_id == para_id.clone() {
            if let Err(subxt::Error::Runtime(dispatch_err)) = reserve(rococo_api.clone()).await {
                eprintln!(
                    "Could not dispatch the call to reserve the para_id: {}",
                    dispatch_err
                );
            }
        } else if next_para_id < para_id {
            println!(
                "Error: ParaId: {} is not reserved and is not the next free para id",
                args.para_id.clone()
            );
            return Ok(());
        }
    }

    // Initialise an empty call buffer
    let mut call_buffer: Vec<Call> = Vec::<Call>::new();

    // If the ParaID is not registered register it with sudo
    let is_registered = is_registered(rococo_api.clone(), para_id.clone()).await;
    if !is_registered.unwrap() {
        // Add call to send funds to `manager_account`
        call_buffer.push(create_force_transfer_call(manager_account.clone()).unwrap());

        let genesis_head = get_file_content(args.path_genesis_head).await;
        let validation_code = get_file_content(args.path_validation_code).await;

        let genesis_bytes = genesis_head.as_bytes().to_vec();
        let validation_code_bytes = parse_validation_code(validation_code);

        call_buffer.push(
            create_force_register_call(
                para_id.clone(),
                manager_account,
                genesis_bytes,
                validation_code_bytes,
            )
            .unwrap(),
        );
    }

    // Add call to send funds to paras sovereign account
    call_buffer.push(
        create_force_transfer_call(calculate_sovereign_account::<Pair>(para_id.clone()).unwrap())
            .unwrap(),
    );

    // Add call to schedule assigning a slot to the given para
    call_buffer.push(create_scheduled_assign_slots_call(para_id.clone(), is_perm_slot).unwrap());

    // Add call to schedule removing the manager lock from the given para
    call_buffer.push(create_scheduled_remove_lock_call(para_id).unwrap());

    // Get the batched call based on the calls present in buffer
    let batch_call = create_batch_all_call(call_buffer).unwrap();

    // Create a SUDO call
    let sudo_call = create_sudo_call(batch_call).unwrap();

    // Sign and send batch_call to the network
    if let Err(subxt::Error::Runtime(dispatch_err)) =
        sign_and_send_proxy_call(rococo_api, sudo_call).await
    {
        eprintln!("Could not dispatch the call: {}", dispatch_err);
    }
    Ok(())
}
