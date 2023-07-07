use std::{fs, path::PathBuf};
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};
use scale::Encode;
use sp_core::{crypto::{Ss58AddressFormatRegistry, Ss58Codec}, sr25519::Pair};
use sp_runtime::{MultiSigner};

#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}
use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;

use crate::calls::{force_register, force_transfer, remove_lock, schedule_assign_slots};
use crate::query::{maybe_leases, paras_registered};

pub enum Chain {
    DOT,
    KSM,
    ROC,
}

pub type Api = OnlineClient<PolkadotConfig>;

// Returns if the passed para_id is applicable for a permanent slot in Rococo
pub async fn needs_perm_slot(para_id: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let polkadot_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io:443").await?;
    let kusama_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://kusama-rpc.polkadot.io:443").await?;

    let lease_polkadot = maybe_leases(polkadot_api, Chain::DOT, para_id).await;

    let lease_kusama = maybe_leases(kusama_api, Chain::KSM, para_id).await;

    if lease_kusama.unwrap() || lease_polkadot.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Returns if the passed para_id already has a slot in Rococo
pub async fn has_slot_in_rococo(para_id: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    let lease_rococo = maybe_leases(rococo_api, Chain::ROC, para_id).await;

    if lease_rococo.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Check if the parachain is registerd  in Rococo
pub async fn is_registered(para_id: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    let is_registered_in_rococo = paras_registered(rococo_api, para_id).await;
    if is_registered_in_rococo.unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

// Force the Register parachain
pub async fn register(
    para_id: u32,
    manager_account: AccountId32,
    path_genesis_head: PathBuf,
    path_validation_code: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let validation_code = fs::read_to_string(path_validation_code)
        .expect("Should have been able to read the validation code file");
    let genesis_head =
        fs::read(path_genesis_head).expect("Should have been able to read the genesis file");

    let rococo_api = get_rococo_uri().await;

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
    para_id: u32,
    is_permanent_slot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    schedule_assign_slots(rococo_api, para_id, is_permanent_slot).await
}

// Fund the parachain manager account
pub async fn fund_parachain_manager(
    account_manager: AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    force_transfer(rococo_api, account_manager).await
}

// Fund the parachain manager account
pub async fn fund_sovereign_account(
    para_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    let sovereign_account_str = calculate_sovereign_account::<Pair>(para_id)?;
    let sovereign_account: AccountId32 = sovereign_account_str.parse().unwrap();
    force_transfer(rococo_api, sovereign_account).await
}

// Remove a manager lock for para_id.
pub async fn remove_parachain_lock(para_id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let rococo_api = get_rococo_uri().await;
    remove_lock(rococo_api, para_id).await
}

fn parse_validation_code(validation_code: String) -> Vec<u8> {
    let mut parsed_validation_code = validation_code;
    // Remove the 0x
    parsed_validation_code.remove(0);
    parsed_validation_code.remove(0);
    // Decode the hex to bytes
    hex::decode(parsed_validation_code).expect("Decoding failed")
}

async fn get_rococo_uri() -> OnlineClient<PolkadotConfig> {
    let uri = std::env::var("ROCOCO_URI").unwrap_or("ws://127.0.0.1:9944".to_string());
    let rococo_api = OnlineClient::<PolkadotConfig>::from_url(uri)
        .await
        .expect("Connection to Rococo failed");
    rococo_api
}


fn calculate_sovereign_account<Pair>(
	para_id: u32,
) -> Result<String, Box<dyn std::error::Error>>
where
	Pair: sp_core::Pair,
	Pair::Public: Into<MultiSigner>,
{
    let id = RococoId(para_id);
    let prefix = hex::encode("para");
    let encoded_id = hex::encode(id.encode());
    let encoded_key = "0x".to_owned() + &prefix + &encoded_id;
    let public_str = format!("{:0<width$}", encoded_key, width = 64 + 2);

    let public = array_bytes::hex2bytes(&public_str).expect("Failed to convert hex to bytes");
	let public_key = Pair::Public::try_from(&public)
        .map_err(|_| "Failed to construct public key from given hex")?;

    Ok(public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into()))
}
