use subxt::{utils::AccountId32};
use subxt::{OnlineClient, PolkadotConfig};
use crate::utils::{get_signer, get_sudo_account};

// #[subxt::subxt(runtime_metadata_path = "metadata/rococo_metadata.scale")]
// pub mod rococo {}
#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}

use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;
use rococo::runtime_types::polkadot_parachain::primitives::{HeadData, ValidationCode};
use rococo::runtime_types::polkadot_runtime_common::assigned_slots::SlotLeasePeriodStart::Current;

pub type Call = rococo::runtime_types::rococo_runtime::RuntimeCall;
type RegistrarCall = rococo::runtime_types::polkadot_runtime_common::paras_registrar::pallet::Call;
type AssignSlotsCall = rococo::runtime_types::polkadot_runtime_common::assigned_slots::pallet::Call;
type SchedulerCall = rococo::runtime_types::pallet_scheduler::pallet::Call;
type BalancesCall = rococo::runtime_types::pallet_balances::pallet::Call;
type UtilityCall = rococo::runtime_types::pallet_utility::pallet::Call;
type SudoCall = rococo::runtime_types::pallet_sudo::pallet::Call;

const BLOCKS_SCHEDULED: u32 = 1205; // 2 epochs (600*2) + 5 blocks of margin
const REGISTER_DEPOSIT: u128 = 10_000;
const FUNDS: u128 = 10_000_000_000_000; // 10 UNITS

//
// Create batch call out of the given calls
//
pub fn create_batch_all_call(
    calls: Vec<Call>,
) -> Result<Call, Box<dyn std::error::Error>> {

    let batch_call = Call::Utility(
        UtilityCall::batch_all { calls: calls }
    );

    Ok(batch_call)
}

//
// Fund the parachain manager from the faucet address using sudo
//
pub fn create_force_transfer_call(
    account_dest: AccountId32,
) -> Result<Call, Box<dyn std::error::Error>> {
    let faucet_address =
        std::env::var("FAUCET_ADDRESS").expect("Error: No Faucer Address provided");
    let account_source: AccountId32 = faucet_address.parse().unwrap();

    let call = Call::Balances(BalancesCall::force_transfer {
        source: account_source.into(),
        dest: account_dest.into(),
        value: FUNDS,
    });

    Ok(call)
}

//
// Register the parachain with sudo
//
pub fn create_force_register_call(
    para_id: u32,
    manager_account: AccountId32,
    genesis_head: Vec<u8>,
    validation_code: Vec<u8>,
) -> Result<Call, Box<dyn std::error::Error>> {
    let call = Call::Registrar(RegistrarCall::force_register {
        who: manager_account,
        deposit: REGISTER_DEPOSIT,
        id: RococoId(para_id),
        genesis_head: HeadData(genesis_head),
        validation_code: ValidationCode(validation_code),
    });

    Ok(call)
}

//
// Creates a call to schedule an assign perm / tmp slot call
//
pub fn create_scheduled_assign_slots_call(
    para_id: u32,
    is_permanent_slot: bool,
) -> Result<Call, Box<dyn std::error::Error>> {
    // Temporary slots by default, and if is_permanent_slot is true, then permanent slots
    let mut call = Call::AssignedSlots(AssignSlotsCall::assign_temp_parachain_slot {
        id: RococoId(para_id),
        lease_period_start: Current,
    });
    if is_permanent_slot {
        call = Call::AssignedSlots(AssignSlotsCall::assign_perm_parachain_slot {
            id: RococoId(para_id),
        });
    }

    let scheduled_call = Call::Scheduler(SchedulerCall::schedule_after {
        after: BLOCKS_SCHEDULED,
        maybe_periodic: None,
        priority: 0,
        call: Box::new(call),
    });

    Ok(scheduled_call)
}

//
// Creates a call to remove the manager lock from the given para
//
pub fn create_scheduled_remove_lock_call(
    para_id: u32,
) -> Result<Call, Box<dyn std::error::Error>> {
    
    let call = Call::Registrar(RegistrarCall::remove_lock {
        para: RococoId(para_id),
    });

    let scheduled_call = Call::Scheduler(SchedulerCall::schedule_after {
        after: BLOCKS_SCHEDULED * 2,
        maybe_periodic: None,
        priority: 0,
        call: Box::new(call),
    });

    Ok(scheduled_call)
}

//
// Creates a sudo call wrapping the given call
//
pub fn create_sudo_call(
    call: Call,
) -> Result<Call, Box<dyn std::error::Error>> {
    
    let sudo_call = Call::Sudo(SudoCall::sudo {
        call: Box::new(call),
    });

    Ok(sudo_call)
}

//
// Sign and send the passed call and waits for
//
pub async fn sign_and_send_proxy_call(
    api: OnlineClient<PolkadotConfig>,
    call: Call,
) ->  Result<(), Box<dyn std::error::Error>> {

    let utx = rococo::tx().proxy().proxy(
        subxt::utils::MultiAddress::Id(get_sudo_account()),
        None,
        call
    );

    api.tx()
        .sign_and_submit_then_watch_default(&utx, &get_signer())
        .await?
        .wait_for_in_block()
        .await?
        .wait_for_success()
        .await?
        .has::<rococo::sudo::events::Sudid>()?;

    Ok(())
}
