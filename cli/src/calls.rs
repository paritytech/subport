use crate::helper::Api;
use subxt::{tx::PairSigner, utils::AccountId32, PolkadotConfig};
use sp_core::Pair;

// #[subxt::subxt(runtime_metadata_path = "metadata/rococo_metadata.scale")]
// pub mod rococo {}
#[subxt::subxt(runtime_metadata_path = "metadata/local_metadata.scale")]
pub mod rococo {}

use rococo::runtime_types::polkadot_parachain::primitives::Id as RococoId;
use rococo::runtime_types::polkadot_parachain::primitives::{HeadData, ValidationCode};

type Call = rococo::runtime_types::rococo_runtime::Call;
type RegistrarCall = rococo::runtime_types::polkadot_runtime_common::paras_registrar::pallet::Call;

//
// Register the parachain with sudo
//
pub async fn force_register(
    api: Api,
    para_id: u32,
    account_manager: AccountId32,
    genesis_head: String,
    validation_code: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice = get_signer();

    let call = Call::Registrar(RegistrarCall::force_register {
        who: account_manager,
        deposit: 10_000,
        id: RococoId(para_id),
        genesis_head: HeadData(genesis_head.into()),
        validation_code: ValidationCode(validation_code.into()),
    });

    let tx = rococo::tx().sudo().sudo(call);

    api.tx()
        .sign_and_submit_then_watch_default(&tx, &alice)
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
