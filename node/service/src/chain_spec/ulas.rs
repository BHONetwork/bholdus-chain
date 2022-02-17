use crate::chain_spec::{authority_keys_from_seed, get_account_id_from_seed, get_from_seed};
use beefy_primitives::crypto::AuthorityId as BeefyId;
use bholdus_primitives::{
    AccountId, Balance, CurrencyId, Signature, TokenInfo, TokenSymbol, TradingPair,
};
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{config::TelemetryEndpoints, ChainType, Properties};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};
use ulas_runtime::{
    opaque::SessionKeys, Aura, AuraConfig, AuthorityDiscoveryConfig, BalancesConfig, BeefyConfig,
    BridgeNativeTransferConfig, CouncilConfig, EVMConfig, EthereumConfig, GenesisAccount,
    GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, SessionConfig, StakerStatus,
    StakingConfig, SudoConfig, BholdusSupportNFTConfig, SystemConfig, TokensConfig, BHO, MAX_NOMINATIONS,
    TOKEN_DECIMALS, TOKEN_SYMBOL, WASM_BINARY,
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
pub fn get_properties() -> Properties {
    let mut properties = Properties::new();

    properties.insert("ss58Format".into(), ulas_runtime::SS58Prefix::get().into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());

    properties
}

fn session_keys(
    grandpa: GrandpaId,
    aura: AuraId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
    beefy: BeefyId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        aura,
        im_online,
        authority_discovery,
        beefy,
    }
}

pub fn config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../../res/ulas/ulas.json")[..])
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let constants = Constants::new();

    Ok(ChainSpec::from_genesis(
        // Name
        "Ulas Development",
        // ID
        "ulas-dev",
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
        "Ulas Local",
        // ID
        "ulas-local",
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

pub fn production_sample_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let constants = Constants::new();

    Ok(ChainSpec::from_genesis(
        // Name
        "Ulas Production",
        // ID
        "ulas-prod",
        ChainType::Live,
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
        AuraId,
        ImOnlineId,
        AuthorityDiscoveryId,
        BeefyId,
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
                        session_keys(
                            x.2.clone(),
                            x.3.clone(),
                            x.4.clone(),
                            x.5.clone(),
                            x.6.clone(),
                        ),
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
        aura: AuraConfig {
            authorities: vec![],
        },
        im_online: ImOnlineConfig { keys: vec![] },
        authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        beefy: BeefyConfig {
            authorities: vec![],
        },
        treasury: Default::default(),
        tokens: TokensConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .filter_map(|(currency_id, account_id, balance, is_native_currency)| {
                    if !is_native_currency {
                        Some((account_id, balance))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        },
        bholdus_support_nft: BholdusSupportNFTConfig { tokens: vec![] },
        bridge_native_transfer: Default::default(),
        evm: EVMConfig {
            accounts: {
                // Prefund the "Gerald" account
                let mut accounts = std::collections::BTreeMap::new();
                accounts.insert(
                    H160::from_slice(&hex_literal::hex!(
                        "fF6a5C321D1AB7B48a39E62cE5de4b0E87EDc828"
                    )),
                    GenesisAccount {
                        nonce: U256::zero(),
                        // Using a larger number, so I can tell the accounts apart by balance.
                        balance: U256::from(1u64 << 61),
                        code: vec![],
                        storage: std::collections::BTreeMap::new(),
                    },
                );
                accounts
            },
        },
        ethereum: EthereumConfig {},
        base_fee: Default::default(),
        /* dex: DexConfig {
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
        }, */
        /* bsc: BSCConfig {
            genesis_header: serde_json::from_str(r#"{
                "difficulty": "0x2",
                "extraData": "0xd883010100846765746888676f312e31352e35856c696e7578000000fc3ca6b72465176c461afb316ebc773c61faee85a6515daa295e26495cef6f69dfa69911d9d8e4f3bbadb89b29a97c6effb8a411dabc6adeefaa84f5067c8bbe2d4c407bbe49438ed859fe965b140dcf1aab71a93f349bbafec1551819b8be1efea2fc46ca749aa14430b3230294d12c6ab2aac5c2cd68e80b16b581685b1ded8013785d6623cc18d214320b6bb6475970f657164e5b75689b64b7fd1fa275f334f28e1872b61c6014342d914470ec7ac2975be345796c2b7ae2f5b9e386cd1b50a4550696d957cb4900f03a8b6c8fd93d6f4cea42bbb345dbc6f0dfdb5bec739bb832254baf4e8b4cc26bd2b52b31389b56e98b9f8ccdafcc39f3c7d6ebf637c9151673cbc36b88a6f79b60359f141df90a0c745125b131caaffd12b8f7166496996a7da21cf1f1b04d9b3e26a3d077be807dddb074639cd9fa61b47676c064fc50d62cce2fd7544e0b2cc94692d4a704debef7bcb61328e2d3a739effcd3a99387d015e260eefac72ebea1e9ae3261a475a27bb1028f140bc2a7c843318afdea0a6e3c511bbd10f4519ece37dc24887e11b55dee226379db83cffc681495730c11fdde79ba4c0c0670403d7dfc4c816a313885fe04b850f96f27b2e9fd88b147c882ad7caf9b964abfe6543625fcca73b56fe29d3046831574b0681d52bf5383d6f2187b6276c100",
                "gasLimit": "0x38ff37a",
                "gasUsed": "0x1364017",
                "logsBloom": "0x2c30123db854d838c878e978cd2117896aa092e4ce08f078424e9ec7f2312f1909b35e579fb2702d571a3be04a8f01328e51af205100a7c32e3dd8faf8222fcf03f3545655314abf91c4c0d80cea6aa46f122c2a9c596c6a99d5842786d40667eb195877bbbb128890a824506c81a9e5623d4355e08a16f384bf709bf4db598bbcb88150abcd4ceba89cc798000bdccf5cf4d58d50828d3b7dc2bc5d8a928a32d24b845857da0b5bcf2c5dec8230643d4bec452491ba1260806a9e68a4a530de612e5c2676955a17400ce1d4fd6ff458bc38a8b1826e1c1d24b9516ef84ea6d8721344502a6c732ed7f861bb0ea017d520bad5fa53cfc67c678a2e6f6693c8ee",
                "miner": "0xe9ae3261a475a27bb1028f140bc2a7c843318afd",
                "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "nonce": "0x0000000000000000",
                "number": "0x7594c8",
                "parentHash": "0x5cb4b6631001facd57be810d5d1383ee23a31257d2430f097291d25fc1446d4f",
                "receiptsRoot": "0x1bfba16a9e34a12ff7c4b88be484ccd8065b90abea026f6c1f97c257fdb4ad2b",
                "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                "stateRoot": "0xa6cd7017374dfe102e82d2b3b8a43dbe1d41cc0e4569f3dc45db6c4e687949ae",
                "timestamp": "0x60ac7137",
                "transactionsRoot": "0x657f5876113ac9abe5cf0460aa8d6b3b53abfc336cea4ab3ee594586f8b584ca"
            }"#).unwrap()
        }, */
    }
}
