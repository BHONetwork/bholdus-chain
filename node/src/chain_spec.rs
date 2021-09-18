use bholdus_primitives::{AccountId, Balance, Signature};
use bholdus_runtime::{
    opaque::SessionKeys, AuraConfig, AuthorityDiscoveryConfig, BalancesConfig, ContractsConfig,
    CouncilConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, SessionConfig,
    StakerStatus, StakingConfig, SudoConfig, SystemConfig, BABE_GENESIS_EPOCH_CONFIG, BHO,
    MAX_NOMINATIONS, TOKEN_DECIMALS, TOKEN_SYMBOL, WASM_BINARY,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{config::TelemetryEndpoints, ChainType, Properties};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "bho";
/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn get_properties() -> Properties {
    let mut properties = Properties::new();

    properties.insert(
        "ss58Format".into(),
        bholdus_runtime::SS58Prefix::get().into(),
    );
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

    properties
}

fn session_keys(
    grandpa: GrandpaId,
    aura: AuraId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        aura,
        im_online,
        authority_discovery,
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    AuraId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<AuraId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Bholdus-Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                vec![],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        1_000 * BHO,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        10_000_000_000 * BHO,
                    ),
                ],
                10_000 * BHO,
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        Some(
            TelemetryEndpoints::new(vec![(
                String::from("wss://telemetry.polkadot.io/submit/"),
                0,
            )])
            .unwrap(),
        ),
        // Protocol ID
        Some(DEFAULT_PROTOCOL_ID),
        // Properties
        Some(get_properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Bholdus-Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                vec![],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        10_000_000_000 * BHO,
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        10_000_000 * BHO,
                    ),
                ],
                1000 * BHO,
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        Some(
            TelemetryEndpoints::new(vec![(
                String::from("wss://telemetry.polkadot.io/submit/"),
                0,
            )])
            .unwrap(),
        ),
        // Protocol ID
        Some(DEFAULT_PROTOCOL_ID),
        // Properties
        Some(get_properties()),
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        AuraId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    initial_nominators: Vec<AccountId>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, Balance)>,
    stash: Balance,
    enable_println: bool,
) -> GenesisConfig {
    // stakers: all validators and nominators.
    let mut rng = rand::thread_rng();
    let stakers = initial_authorities
        .iter()
        .map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
        .chain(initial_nominators.iter().map(|x| {
            use rand::{seq::SliceRandom, Rng};
            let limit = (MAX_NOMINATIONS as usize).min(initial_authorities.len());
            let count = rng.gen::<usize>() % limit;
            let nominations = initial_authorities
                .as_slice()
                .choose_multiple(&mut rng, count)
                .into_iter()
                .map(|choice| choice.0.clone())
                .collect::<Vec<_>>();
            (
                x.clone(),
                x.clone(),
                stash,
                StakerStatus::Nominator(nominations),
            )
        }))
        .collect::<Vec<_>>();

    GenesisConfig {
        frame_system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            balances: endowed_accounts.iter().cloned().collect(),
        },
        pallet_indices: IndicesConfig { indices: vec![] },
        pallet_session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        pallet_staking: StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            stakers,
            ..Default::default()
        },
        pallet_collective_Instance1: CouncilConfig::default(),
        pallet_sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        pallet_aura: AuraConfig {
            authorities: vec![],
        },
        pallet_im_online: ImOnlineConfig { keys: vec![] },
        pallet_authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
        pallet_grandpa: GrandpaConfig {
            authorities: vec![],
        },
        pallet_treasury: Default::default(),
        pallet_contracts: ContractsConfig {
            // println should only be enabled on development chains
            current_schedule: pallet_contracts::Schedule::default().enable_println(enable_println),
        },
    }
}
