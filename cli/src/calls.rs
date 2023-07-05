use crate::helper::Api;
use sp_core::Pair;
use subxt::{tx::PairSigner, utils::AccountId32, PolkadotConfig};

// #[subxt::subxt(runtime_metadata_path = "metadata/rococo_metadata.scale")]
// pub mod rococo {}
#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}

const BLOCKS_SCHEDULED: u32 = 20;
const DEPOSIT_REGISTER: u128 = 10_000;
const FUNDS_MANAGER: u128 = 10_000_000_000_000; // 10 UNITS

use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;
use rococo::runtime_types::polkadot_parachain::primitives::{HeadData, ValidationCode};
use rococo::runtime_types::polkadot_runtime_common::assigned_slots::SlotLeasePeriodStart::Current;

type Call = rococo::runtime_types::rococo_runtime::RuntimeCall;
type RegistrarCall = rococo::runtime_types::polkadot_runtime_common::paras_registrar::pallet::Call;
type AssignSlotsCall = rococo::runtime_types::polkadot_runtime_common::assigned_slots::pallet::Call;
type SchedulerCall = rococo::runtime_types::pallet_scheduler::pallet::Call;
type BalancesCall = rococo::runtime_types::pallet_balances::pallet::Call;

//
// Register the parachain with sudo
//
pub async fn force_register(
    api: Api,
    para_id: u32,
    account_manager: AccountId32,
    genesis_head: Vec<u8>,
    validation_code: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = get_signer();
    let call = Call::Registrar(RegistrarCall::force_register {
        who: account_manager,
        deposit: DEPOSIT_REGISTER,
        id: RococoId(para_id),
        genesis_head: HeadData(genesis_head),
        validation_code: ValidationCode(validation_code),
    });

    let tx = rococo::tx().sudo().sudo(call);

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &root)
        .await?
        .wait_for_finalized_success()
        .await?
        .has::<rococo::sudo::events::Sudid>()?;
    Ok(())
}

//
// Schedule the assign slots into a parachain with sudo
//
pub async fn schedule_assign_slots(
    api: Api,
    para_id: u32,
    is_permanent_slot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = get_signer();

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

    let tx = rococo::tx().sudo().sudo(scheduled_call);

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &root)
        .await?
        .wait_for_finalized_success()
        .await?
        .has::<rococo::sudo::events::Sudid>()?;
    Ok(())
}

//
// Fund the parachain manager from the faucet address using sudo
//
pub async fn force_transfer(
    api: Api,
    account_dest: AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = get_signer();
    let faucet_address =
        std::env::var("FAUCET_ADDRESS").expect("Error: No Faucer Address provided");
    let account_source: AccountId32 = faucet_address.parse().unwrap();

    let call = Call::Balances(BalancesCall::force_transfer {
        source: account_source.into(),
        dest: account_dest.into(),
        value: FUNDS_MANAGER,
    });

    let tx = rococo::tx().sudo().sudo(call);

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &root)
        .await?
        .wait_for_finalized_success()
        .await?
        .has::<rococo::sudo::events::Sudid>()?;
    Ok(())
}

//
// Remove a manager lock from a para. This will allow the manager of a
// previously locked para to deregister or swap a para without using governance.
//
pub async fn remove_lock(api: Api, para_id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let root = get_signer();

    let call = Call::Registrar(RegistrarCall::remove_lock {
        para: RococoId(para_id),
    });

    let tx = rococo::tx().sudo().sudo(call);

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &root)
        .await?
        .wait_for_finalized_success()
        .await?
        .has::<rococo::sudo::events::Sudid>()?;
    Ok(())
}

fn get_signer() -> PairSigner<PolkadotConfig, sp_core::sr25519::Pair> {
    let mnemonic_phrase = std::env::var("SEED").expect("Error: No SEED provided");
    let pair = sp_core::sr25519::Pair::from_string(&mnemonic_phrase, None).unwrap();
    PairSigner::new(pair)
}
