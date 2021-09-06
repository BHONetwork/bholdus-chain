use bholdus_primitives::{
    AccountId, Balance, CurrencyId, Signature, TokenInfo, TokenSymbol, TradingPair,
};
use bholdus_runtime::{
    opaque::SessionKeys, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig,
    ChainBridgeTransferConfig, CouncilConfig, DexConfig, GenesisConfig, GrandpaConfig,
    ImOnlineConfig, IndicesConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
    SystemConfig, TokensConfig, BABE_GENESIS_EPOCH_CONFIG, BHO, MAX_NOMINATIONS, TOKEN_DECIMALS,
    TOKEN_SYMBOL, WASM_BINARY,
};
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{config::TelemetryEndpoints, ChainType, Properties};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
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

struct Constants {
    pub BHO_CURRENCY: CurrencyId,
    pub BNB_CURRENCY: CurrencyId,
    pub DOT_CURRENCY: CurrencyId,
    pub BHO_BNB_PAIR: TradingPair,
    pub BNB_DOT_PAIR: TradingPair,
    pub BHO_BNB_SHARE_CURRENCY: CurrencyId,
    pub BNB_DOT_SHARE_CURRENCY: CurrencyId,
}

impl Constants {
    fn new() -> Constants {
        let BHO_CURRENCY: CurrencyId = CurrencyId::Token(TokenSymbol::Native);
        let BNB_CURRENCY: CurrencyId = CurrencyId::Token(TokenSymbol::Token(TokenInfo { id: 1 }));
        let DOT_CURRENCY: CurrencyId = CurrencyId::Token(TokenSymbol::Token(TokenInfo { id: 2 }));
        let BHO_BNB_PAIR: TradingPair =
            TradingPair::from_currency_ids(BNB_CURRENCY, BHO_CURRENCY).unwrap();
        let BNB_DOT_PAIR: TradingPair =
            TradingPair::from_currency_ids(BNB_CURRENCY, DOT_CURRENCY).unwrap();
        Constants {
            BHO_CURRENCY,
            BNB_CURRENCY,
            DOT_CURRENCY,
            BHO_BNB_PAIR,
            BNB_DOT_PAIR,
            BHO_BNB_SHARE_CURRENCY: BHO_BNB_PAIR.dex_share_currency_id(),
            BNB_DOT_SHARE_CURRENCY: BNB_DOT_PAIR.dex_share_currency_id(),
        }
    }
}

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
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
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
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let constants = Constants::new();

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
                        constants.BHO_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        1_000_000_000 * BHO,
                        true,
                    ),
                    (
                        constants.BHO_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        10_000_000_000 * BHO,
                        true,
                    ),
                    (
                        constants.BNB_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        10_000_000 * BHO,
                        false,
                    ),
                    (
                        constants.DOT_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        10_000_000_000 * BHO,
                        false,
                    ),
                ],
                vec![],
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
    let constants = Constants::new();

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
                        constants.BHO_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                        10_000_000_000 * BHO,
                        true,
                    ),
                    (
                        constants.BHO_CURRENCY,
                        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        10_000_000 * BHO,
                        true,
                    ),
                ],
                vec![],
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
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    initial_nominators: Vec<AccountId>,
    root_key: AccountId,
    endowed_accounts: Vec<(CurrencyId, AccountId, Balance, bool)>,
    initial_dex_liquidity_pairs: Vec<(AccountId, (CurrencyId, CurrencyId), (Balance, Balance))>,
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
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .filter_map(|(currency_id, account_id, balance, is_native_currency)| {
                    if is_native_currency {
                        Some((account_id, balance))
                    } else {
                        None
                    }
                })
                .collect(),
        },
        indices: IndicesConfig { indices: vec![] },
        session: SessionConfig {
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
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            stakers,
            ..Default::default()
        },
        council: CouncilConfig::default(),
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        im_online: ImOnlineConfig { keys: vec![] },
        authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        treasury: Default::default(),
        // contracts: ContractsConfig {
        // println should only be enabled on development chains
        // current_schedule: pallet_contracts::Schedule::default().enable_println(enable_println),
        // },
        chain_bridge_transfer: ChainBridgeTransferConfig {},
        tokens: TokensConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .filter_map(|(currency_id, account_id, balance, is_native_currency)| {
                    if !is_native_currency {
                        Some((account_id, currency_id, balance))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        },
        dex: DexConfig {
            initial_provisioning_trading_pairs: vec![],
            initial_enabled_trading_pairs: initial_dex_liquidity_pairs
                .iter()
                .cloned()
                .map(|(_, (currency_id_0, currency_id_1), ..)| {
                    TradingPair::from_currency_ids(currency_id_0, currency_id_1).unwrap()
                })
                .collect(),
            initial_added_liquidity_pools: initial_dex_liquidity_pairs
                .iter()
                .cloned()
                .map(
                    |(account_id, (currency_id_0, currency_id_1), (amount_0, amount_1))| {
                        let trading_pair =
                            TradingPair::from_currency_ids(currency_id_0, currency_id_1).unwrap();
                        let pair_and_amount = if currency_id_0 == trading_pair.first() {
                            vec![(trading_pair, (amount_0, amount_1))]
                        } else {
                            vec![(trading_pair, (amount_1, amount_0))]
                        };
                        (account_id, pair_and_amount)
                    },
                )
                .collect(),
        },
    }
}
