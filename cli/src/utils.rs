use scale::Encode;
use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_runtime::MultiSigner;

#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}
use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;

pub fn parse_validation_code(validation_code: String) -> Vec<u8> {
    let mut parsed_validation_code = validation_code;
    // Remove "0x" from validation_code
    let parsed_validation_code = &validation_code[2..];
    // Decode the hex to bytes
    hex::decode(parsed_validation_code).expect("Decoding failed")
}

pub fn calculate_sovereign_account<Pair>(para_id: u32) -> Result<String, Box<dyn std::error::Error>>
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
