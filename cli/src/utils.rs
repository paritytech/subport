use scale::Encode;
use sp_core::{
    crypto::{Ss58AddressFormatRegistry, Ss58Codec},
    Pair,
};
use sp_runtime::MultiSigner;
use subxt::{tx::PairSigner, utils::AccountId32, OnlineClient, PolkadotConfig};

use crate::query::{maybe_leases, paras_registered};

// Rococo types
#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}
use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;

// Different chains we can connect to
pub enum Chain {
    DOT,
    KSM,
    ROC,
}

pub fn get_signer() -> PairSigner<PolkadotConfig, sp_core::sr25519::Pair> {
    let mnemonic_phrase = std::env::var("SEED").expect("Error: No SEED provided");
    let pair = sp_core::sr25519::Pair::from_string(&mnemonic_phrase, None).unwrap();
    PairSigner::new(pair)
}

pub fn parse_validation_code(validation_code: String) -> Vec<u8> {
    // Remove "0x" from validation_code
    let parsed_validation_code = &validation_code[2..];
    // Decode the hex to bytes
    hex::decode(parsed_validation_code).expect("Decoding failed")
}

pub fn calculate_sovereign_account<Pair>(para_id: u32) -> Result<AccountId32, Box<dyn std::error::Error>>
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
    let to_parse = public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into());
    Ok(to_parse.parse().unwrap())
}

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