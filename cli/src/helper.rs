use sp_core::sr25519::Pair;
use std::{fs, path::PathBuf};
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

use crate::calls::{force_register, force_transfer, remove_lock, schedule_assign_slots};
use crate::query::{maybe_leases, paras_registered};
use crate::utils::{calculate_sovereign_account, parse_validation_code};

pub enum Chain {
    DOT,
    KSM,
    ROC,
}

pub type Api = OnlineClient<PolkadotConfig>;

// Returns if the passed para_id already has a slot in Rococo
pub async fn has_slot_in_rococo(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let lease_rococo = maybe_leases(rococo_api, Chain::ROC, para_id).await;

    if lease_rococo.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Check if the parachain is registerd  in Rococo
pub async fn is_registered(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let is_registered_in_rococo = paras_registered(rococo_api, para_id).await;
    if is_registered_in_rococo.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Force the Register parachain
pub async fn register(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
    manager_account: AccountId32,
    path_genesis_head: PathBuf,
    path_validation_code: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let validation_code = fs::read_to_string(path_validation_code)
        .expect("Should have been able to read the validation code file");
    let genesis_head =
        fs::read(path_genesis_head).expect("Should have been able to read the genesis file");

    force_register(
        rococo_api,
        para_id,
        manager_account,
        genesis_head,
        parse_validation_code(validation_code),
    )
    .await
}

// Force the Register parachain
pub async fn assign_slots(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
    is_permanent_slot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    schedule_assign_slots(rococo_api, para_id, is_permanent_slot).await
}

// Fund the parachain manager account
pub async fn fund_parachain_manager(
    rococo_api: OnlineClient<PolkadotConfig>,
    account_manager: AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    force_transfer(rococo_api, account_manager).await
}

// Fund the parachain manager account
pub async fn fund_sovereign_account(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let sovereign_account_str = calculate_sovereign_account::<Pair>(para_id)?;
    let sovereign_account: AccountId32 = sovereign_account_str.parse().unwrap();
    force_transfer(rococo_api, sovereign_account).await
}

// Remove a manager lock for para_id.
pub async fn remove_parachain_lock(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    remove_lock(rococo_api, para_id).await
}

// Returns if the passed para_id is applicable for a permanent slot in Rococo
pub async fn needs_perm_slot(
    polkadot_api: OnlineClient<PolkadotConfig>,
    kusama_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let lease_polkadot = maybe_leases(polkadot_api, Chain::DOT, para_id).await;

    let lease_kusama = maybe_leases(kusama_api, Chain::KSM, para_id).await;

    if lease_kusama.unwrap() || lease_polkadot.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}
