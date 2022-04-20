use crate::chain_spec::{authority_keys_from_seed, get_account_id_from_seed};
use beefy_primitives::crypto::AuthorityId as BeefyId;
use common_primitives::{AccountId, Balance};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{config::TelemetryEndpoints, ChainType, Properties};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::Perbill;
use ulas_runtime::{
	AuraConfig, AuthorityDiscoveryConfig, BalancesConfig, BeefyConfig, BholdusSupportNFTConfig,
	CouncilConfig, EVMConfig, EthereumConfig, GenesisAccount, GenesisConfig, GrandpaConfig,
	ImOnlineConfig, IndicesConfig, MaxNominations, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, SudoConfig, SystemConfig, TokensConfig, BHO, TOKEN_DECIMALS, TOKEN_SYMBOL,
	WASM_BINARY,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "bho";
/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
	SessionKeys { grandpa, aura, im_online, authority_discovery, beefy }
}

pub fn config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/ulas/ulas.json")[..])
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000_000 * BHO),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						10_000_000_000 * BHO,
					),
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 10_000_000 * BHO),
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 10_000_000_000 * BHO),
				],
				10_000 * BHO,
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(String::from("wss://telemetry.polkadot.io/submit/"), 0)])
				.unwrap(),
		),
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		// Fork ID
		None,
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
		"Ulas Local",
		// ID
		"ulas-local",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						10_000_000_000 * BHO,
					),
					(get_account_id_from_seed::<sr25519::Public>("Bob//stash"), 10_000_000 * BHO),
				],
				1000 * BHO,
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(String::from("wss://telemetry.polkadot.io/submit/"), 0)])
				.unwrap(),
		),
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		// Fork ID
		None,
		// Properties
		Some(get_properties()),
		// Extensions
		None,
	))
}

pub fn production_sample_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

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
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						10_000_000_000 * BHO,
					),
					(get_account_id_from_seed::<sr25519::Public>("Bob//stash"), 10_000_000 * BHO),
				],
				1000 * BHO,
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(String::from("wss://telemetry.polkadot.io/submit/"), 0)])
				.unwrap(),
		),
		// Protocol ID
		Some(DEFAULT_PROTOCOL_ID),
		// Fork ID
		None,
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
	endowed_accounts: Vec<(AccountId, Balance)>,
	stash: Balance,
	_enable_println: bool,
) -> GenesisConfig {
	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), stash, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.filter_map(|(account_id, balance)| Some((account_id, balance)))
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
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		aura: AuraConfig { authorities: vec![] },
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		beefy: BeefyConfig { authorities: vec![] },
		treasury: Default::default(),
		tokens: TokensConfig { balances: vec![] },
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
	}
}
