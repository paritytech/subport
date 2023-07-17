use sp_core::sr25519::Pair;
use std::{fs, path::PathBuf};
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

use crate::calls::{batch, force_register, schedule_assign_slots, force_transfer};
use crate::query::{maybe_leases, paras_registered};
use crate::utils::{calculate_sovereign_account, parse_validation_code};

pub enum Chain {
    DOT,
    KSM,
    ROC,
}

const FUND_AMMOUNT = 50;

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

// Send funds to the given account
pub async fn fund_account(account: AccountId32) -> Result<(), Box<dyn std::error::Error>> {
    let rococo_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?;
    force_transfer(
        rococo_api,
        AcountId32::from("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        account,
        FUND_AMMOUNT,
    ).await
    
// Batch for various calls: remove parachain lock, fund parachain manager and sovereign account and schedule assign slots
pub async fn batch_calls(
    rococo_api: OnlineClient<PolkadotConfig>,
    para_id: u32,
    manager_account: AccountId32,
    is_permanent_slot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let sovereign_account_str = calculate_sovereign_account::<Pair>(para_id)?;
    let sovereign_account: AccountId32 = sovereign_account_str.parse().unwrap();
    batch(
        rococo_api,
        para_id,
        manager_account,
        sovereign_account,
        is_permanent_slot,
    )
    .await
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

fn parse_validation_code(validation_code: String) -> Vec<u8> {
    // Remove "0x" from validation_code
    let parsed_validation_code = &validation_code[2..];
    // Decode the hex to bytes
    hex::decode(parsed_validation_code).expect("Decoding failed")
}