//! Tokens Module
//!
//! A simple, secure module for dealing with fungible tokens.
//!
//! ## Overview
//!
//! The Tokens module provides functionality for token managenment of fungible token classes
//! with a fixed supply, including:
//!
//! * Token Issuance (Minting)
//! * Token Transferal
//! * Token Freezing
//!
//! To use it in your runtime, you need to implement the tokens [`Config`].
//!
//! The supported dispatchable functions are documented in the [`Call`] enum.
//!
//! ### Terminology
//!
//! * **Admin**: An account ID uniquely privileged to be able to unfreeze (thaw) an account and
//! it's assets
//! * **Token issuance/minting**: The creation of a new token, whose total supply will belong to
//! the account that issues the asset. This is a privileged operation.
//!
//! * **Fungible token**: An token whose units are interchangeable.
//! * **Issuer**: An account ID uniquely privileged to be able to mint a particular class of
//! assets.
//! * **Freezer**: An account ID uniquely privileged to be able to freeze an account from
//! transferring a particular class of tokens.
//!
//! * **Freezing**: Removing the possibility of an unpermissioned transfer of an token from a
//! particular account.
//! * **Non-fungible token**: An token for which each unit has unique characteristics.
//! * **Owner**: An account ID uniquely privileged to be able to destroy a particular token class,
//! or to set the Issuer, Freezer or Admin of that token class.
//! * **Approval**: The act of allowing an account the permission to transfer some balance of token
//! from approving account into some third-party destination account.
//! * **Sufficiency**: The idea of a minimum-balance of an token being sufficient to allow the
//! account's existence on the system without requiring any other existential-deposit.
//!
//! ## Interface
//!
//! ## Permissionless Functions
//!
//! * `create`: Creates a new asset class, taking the required deposit.
//! * `transfer`: Transfer sender's assets to another account.
//! * `set_metadata`: Set the metadata of an asset class.
//! * `clear_metadata`: Remove the metadata of an asset class.
//! * `set_identity`: Set the associated identity of an asset; a small deposit is reserved if not
//! already taken.
//! * `clear_identity`: Remove an asset's associated identity; the deposit is returned.
//!
//! ### Permissioned Functions
//!
//! * `verify_asset`: Verify the associated identity of an asset.
//!
//! ## Privileged Functions
//! * `destroy`: Destroys an entire asset class; called by the asset class's Owner.
//! * `mint`: Increases the asset balance of an account; called by the asset class's Issuer.
//! * `freeze`: Disallows further `transfer`'s from an accounts; called by the asset class's Admin.
//! * `thaw`: Allows further `transfer`s from an account; called by the asset class's Owner.
//!
//! ### Public Functions
//!
//! * `balance` - Get the asset `id` balance of `who`.
//! * `total_supply` - Get the total supply of an asset `id`.
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use crate::imbalances::{NegativeImbalance, PositiveImbalance};

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
mod imbalances;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

mod extra_mutator;
pub use extra_mutator::*;
mod functions;
mod impl_fungibles;
mod impl_stored_map;
mod types;
pub use types::*;

use sp_runtime::{
	traits::{
		AppendZerosInput, AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub,
		MaybeSerializeDeserialize, Member, One, Saturating, StaticLookup, Zero,
	},
	ArithmeticError, DispatchError, DispatchResult, TokenError,
};
use sp_std::{
	borrow::Borrow,
	collections::btree_set::BTreeSet,
	convert::{Infallible, TryFrom, TryInto},
	marker,
	prelude::*,
	vec::Vec,
};

// Identity
use sp_std::{fmt::Debug, iter::once, ops::Add};

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::{fungibles, DepositConsequence, WithdrawConsequence},
		BalanceStatus as Status, Currency as PalletCurrency, ExistenceRequirement, Get, Imbalance,
		LockableCurrency as PalletLockableCurrency, ReservableCurrency as PalletReservableCurrency,
		SignedImbalance, StoredMap, WithdrawReasons,
	},
	transactional, BoundedVec, PalletId,
};
use std::collections::BTreeMap;

// use frame_support::{
//     dispatch::{DispatchError, DispatchResult},
//     ensure,
// };
use frame_system::{ensure_signed, pallet_prelude::*, Config as SystemConfig};

use bholdus_support::{
	arithmetic::{self, Signed},
	currency::TransferAll,
	BalanceStatus, GetByKey, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
	MultiLockableCurrency, MultiReservableCurrency,
};

pub use pallet::*;
pub use weights::WeightInfo;

impl<Balance: Saturating + Copy + Ord, Extra> AssetBalance<Balance, Extra> {
	/// The total balance in this account including any that is reserved and
	/// ignoring any frozen.
	fn total(&self) -> Balance {
		self.free.saturating_add(self.reserved)
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub balances: Vec<(T::AccountId, T::Balance)>,
	}
	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			GenesisConfig { balances: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			// ensure no duplicates exist.
			// let unique_endowed_accounts = self
			//     .balances
			//     .iter()
			//     .map(|(account_id, _)| (account_id, asset_id))
			//     .collect::<std::collections::BTreeSet<_>>();
			// assert!(
			//     unique_endowed_accounts.len() == self.balances.len(),
			//     "duplicate endowed accounts in genesis."
			// );

			self.balances.iter().for_each(|(account_id, initial_balance)| {
				// assert!(
				//     *initial_balance >= T::ExistentialDeposits::get(&asset_id),
				//     "the balance must be greater than existential deposit.",
				// );
				Pallet::<T, I>::set_genesis(account_id, *initial_balance);
			});
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		/// The amount type, should be signed version of `Balance`
		type Amount: Signed
			+ TryInto<Self::Balance>
			+ TryFrom<Self::Balance>
			+ Parameter
			+ Member
			+ arithmetic::SimpleArithmetic
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;

		/// Identifier for the class of asset.
		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ AtLeast32BitUnsigned;

		/// The currency mechanism.
		type Currency: PalletReservableCurrency<Self::AccountId>;

		/// The origin which may forcibly create or destroy an asset or otherwise alter privileged
		/// attributes.
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The basic amount of funds that must be reserved for an asset.
		type AssetDeposit: Get<DepositBalanceOf<Self, I>>;

		/// The amount held on deposit for a registred identiy
		#[pallet::constant]
		type BasicDeposit: Get<BalanceOf<Self, I>>;

		/// The amount held on deposit per additional field for a registered identity.
		#[pallet::constant]
		type FieldDeposit: Get<BalanceOf<Self, I>>;

		/// The basic ammount of funds that must be reserved when adding metadata to your asset.
		type MetadataDepositBase: Get<DepositBalanceOf<Self, I>>;

		/// The additional funds that must be reserved for the number of bytes you store in your
		/// metadata.
		type MetadataDepositPerByte: Get<DepositBalanceOf<Self, I>>;

		/// The amount of funds that must be reserved when creating a new approval.
		type ApprovalDeposit: Get<DepositBalanceOf<Self, I>>;

		/// The maximum length of a name or symbol stored on-chain.
		type StringLimit: Get<u32>;

		/// The maximum of decimals
		type MaxDecimals: Get<u32>;

		/// A hook to allow a per-asset, per-account minimum balance to be enforced. This must be
		/// respected in all permissionless operations.
		type Freezer: FrozenBalance<Self::AssetId, Self::AccountId, Self::Balance>;

		/// Additional data to be stored with an account's asset balance.
		type Extra: Member + Parameter + Default + MaxEncodedLen;

		/// Maximum number of additional fields that may be stored in an ID. Needed to bound the
		/// I/O required to access an identity, but can be pretty high.
		#[pallet::constant]
		type MaxAdditionalFields: Get<u32>;

		/// Maximum number of registrars allowed in the system. Needed to bound the complexity
		/// of, e.g., updating judgements.
		#[pallet::constant]
		type MaxRegistrars: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The minimum amount required to keep an account.
		type ExistentialDeposits: GetByKey<Self::AssetId, Self::Balance>;
	}

	#[pallet::storage]
	#[pallet::getter(fn assets_blacklist)]
	pub type AssetsBlacklist<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BTreeSet<(Vec<u8>, Vec<u8>)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AssetId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn asset)]
	/// Details of an asset.
	pub(super) type Asset<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T, I>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	/// The number of units of assets held by any given account.
	pub(super) type Account<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		Blake2_128Concat,
		T::AccountId,
		AssetBalance<T::Balance, T::Extra>,
		ValueQuery,
		GetDefault,
		ConstU32<300_000>,
	>;

	#[pallet::storage]
	/// Approved balance transfer. First balance is the amount approved for transfer. Second
	/// is the amount of `T::Currency` reserved for storing this.
	/// First key is the asset ID, second key is the owner and third key is the delegate.
	pub(super) type Approvals<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AssetId>,
			NMapKey<Blake2_128Concat, T::AccountId>, // owner
			NMapKey<Blake2_128Concat, T::AccountId>, // delegate
		),
		Approval<T::Balance, DepositBalanceOf<T, I>>,
		OptionQuery,
	>;

	#[pallet::storage]
	/// Metadata of an asset.
	pub(super) type Metadata<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::AssetId,
		AssetMetadata<DepositBalanceOf<T, I>, BoundedVec<u8, T::StringLimit>>,
		ValueQuery,
		GetDefault,
		ConstU32<300_000>,
	>;

	/// Information that is pertinet to identity the entity behind an account.
	///
	/// TWOX-NOTE: OK - `AccountId` is a secure hash.
	#[pallet::storage]
	#[pallet::getter(fn identity)]
	pub(super) type IdentityOf<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::AssetId, Registration<BalanceOf<T, I>>, OptionQuery>;

	// /// Any liquidity locks of a token type under an account.
	// /// NOTE: Should only be accessed when setting, changing and freeing a lock.
	// #[pallet::storage]
	// #[pallet::getter(fn locks)]
	// pub type Locks<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
	//     _,
	//     Blake2_128Concat,
	//     T::AccountId,
	//     Twox64Concat,
	//     T::AssetId,
	//     BoundedVec<BalanceLock<T::Balance>, T::MaxLocks>,
	//     ValueQuery,
	// >;
	//

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A name was set or reset (which will remove all judgements). \[asset_id]\
		IdentitySet(T::AssetId),
		/// Some asset class was created. \[asset_id, creator, owner\]
		Created(T::AssetId, T::AccountId, T::AccountId),
		/// Some asset class was created and minted. \[asset_id, creator, owner, beneficiary,
		/// metadata\]
		CreateMinted(
			T::AssetId,
			T::AccountId,
			T::AccountId,
			T::AccountId,
			AssetMetadata<DepositBalanceOf<T, I>, BoundedVec<u8, T::StringLimit>>,
		),
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(T::AssetId, T::AccountId, T::Balance),
		/// Some assets were transferred. \[asset_id, owner, total_supply\]
		Transferred(T::AssetId, T::AccountId, T::AccountId, T::Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Burned(T::AssetId, T::AccountId, T::Balance),
		/// Some account `who` was frozen. \[asset_id, who\]
		Frozen(T::AssetId, T::AccountId),
		/// Some account `who` was frozen. \[asset_id, who]\
		Thawed(T::AssetId, T::AccountId),
		/// An account was created with some free balance. \[asset_id, account, free_balance\]
		Endowed(T::AssetId, T::AccountId, T::Balance),
		/// Some asset `asset_id` was frozen. \[asset_id]\
		AssetFrozen(T::AssetId),
		/// Some asset `asset_id` was thawed. \[asset_id]\
		AssetThawed(T::AssetId),
		/// Some asset `asset_id` was verified. \[asset_id]\
		AssetVerified(T::AssetId),
		/// An asset class was destroyed.
		Destroyed(T::AssetId),
		/// Some asset class was force-created. \[asset_id, owner\]
		ForceCreated(T::AssetId, T::AccountId),
		/// New metadata has been set for an asset. \[asset_id, name, symbol, decimals, is_frozen\]
		MetadataSet(T::AssetId, Vec<u8>, Vec<u8>, u8, bool),

		/// Set blacklist. \[name, symbol\]
		BlacklistSet(Vec<u8>, Vec<u8>),

		/// Metadata has been cleared for an asset. \[asset_id\]
		MetadataCleared(T::AssetId),
		/// New identity has been set for an asset. \[asset_id, name\]
		ProfileSet(T::AssetId, Vec<u8>, bool),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Invalid Symbol
		InvalidSymbol,

		/// Invalid Decimals
		InvalidDecimals,

		/// Invalid amount
		ExceedTotalSupply,

		/// Asset belong blacklist
		AssetBlacklist,
		/// No available token ID
		NoAvailableTokenId,
		/// Account balance must be greater than or equal to the transfer amount.
		BalanceLow,
		/// Value too low to create account due to existential deposit
		ExistentialDeposit,
		/// Cannot convert Amount into Balance type
		AmountIntoBalanceFailed,
		/// Failed because liquidity restrictions due to locking
		LiquidityRestrictions,
		/// Failed because the maximum locks was exceeded
		MaxLocksExceeded,
		/// Transfer/payment would kill account
		KeepAlive,
		/// Balance should be non-zero.
		BalanceZero,
		/// The signing account has no permission to do the operation.
		NoPermission,
		/// The given asset ID is unknown.
		Unknown,
		/// The origin account is frozen.
		Frozen,
		/// The asset ID is already taken.
		InUse,
		/// Invalid witness data given.
		BadWitness,
		/// Minimum balance should be non-zero.
		MinBalanceZero,
		/// A mint operation lead to an overflow.
		Overflow,
		/// No provider reference exists to allow a non-zero balance of a non-self-sufficient
		/// asset.
		NoProvider,
		/// Invalid metadata given.
		BadMetadata,
		/// No approval exists that would allow the transfer.
		Unapproved,
		/// The source account would not survive the transfer an it needs to stay alive.
		WouldDie,
		/// Too many additional fields.
		TooManyFields,
	}
	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Issue a new class of fungible assets from a public origin.
		///
		/// This new asset class has no assets initially and its owner is the origin.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Funds of sender are reserved by `AssetDeposit`.
		///
		/// Parameters:
		/// - `id`: The identifier of the new asset. This must not be currently in use to identify
		/// an existing asset.
		/// - `admin`: The admin of this class of assets. The admin is the initial address of each
		/// member of the asset class's admin team
		/// - `min_balance`: The minimum balance of this new asset that any single account must
		/// have. If an account's balance is reduced below this, then it collapses to zero.
		///
		/// Emits `Created` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::create())]
		pub fn create(
			origin: OriginFor<T>,
			admin: <T::Lookup as StaticLookup>::Source,
			min_balance: T::Balance,
		) -> DispatchResult {
			let token_id =
				NextAssetId::<T, I>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T, I>::NoAvailableTokenId)?;
					Ok(current_id)
				})?;

			let owner = ensure_signed(origin)?;
			let admin = T::Lookup::lookup(admin)?;

			ensure!(!min_balance.is_zero(), Error::<T, I>::MinBalanceZero);

			let deposit = T::AssetDeposit::get();
			T::Currency::reserve(&owner, deposit)?;

			Asset::<T, I>::insert(
				token_id,
				AssetDetails {
					owner: owner.clone(),
					issuer: admin.clone(),
					admin: admin.clone(),
					freezer: admin.clone(),
					supply: Zero::zero(),
					deposit,
					min_balance,
					is_sufficient: false,
					accounts: 0,
					sufficients: 0,
					approvals: 0,
					is_frozen: false,
				},
			);
			Self::deposit_event(Event::Created(token_id, owner, admin));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::create_and_mint(name.len() as u32, symbol.len() as u32))]
		pub fn create_and_mint(
			origin: OriginFor<T>,
			admin: <T::Lookup as StaticLookup>::Source,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			beneficiary: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] supply: T::Balance,
			min_balance: T::Balance,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let admin = T::Lookup::lookup(admin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;
			if supply.is_zero() {
				return Ok(());
			}
			ensure!(Self::is_valid_symbol(symbol.clone()), Error::<T, I>::InvalidSymbol);
			ensure!(!min_balance.is_zero(), Error::<T, I>::MinBalanceZero);
			ensure!(
				decimals.clone() <= T::MaxDecimals::get() as u8,
				Error::<T, I>::InvalidDecimals
			);

			let _blacklist =
				AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone()));
			ensure!(
				!AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone())),
				Error::<T, I>::AssetBlacklist
			);

			let bounded_name: BoundedVec<u8, T::StringLimit> = Self::get_name(name.clone())
				.clone()
				.try_into()
				.map_err(|_| Error::<T, I>::BadMetadata)?;
			let bounded_symbol: BoundedVec<u8, T::StringLimit> =
				symbol.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;

			let token_id =
				NextAssetId::<T, I>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T, I>::NoAvailableTokenId)?;
					Ok(current_id)
				})?;

			let deposit = T::AssetDeposit::get();
			T::Currency::reserve(&owner, deposit)?;

			Account::<T, I>::try_mutate(token_id, &beneficiary, |t| -> DispatchResult {
				let new_balance = t.free.saturating_add(supply);
				ensure!(new_balance >= min_balance, TokenError::BelowMinimum);
				if t.free.is_zero() {
					t.sufficient = {
						frame_system::Pallet::<T>::inc_consumers(&beneficiary)
							.map_err(|_| Error::<T, I>::NoProvider)?;
						false
					};
				}
				t.free = new_balance;

				let details = AssetDetails {
					owner: owner.clone(),
					issuer: admin.clone(),
					admin: admin.clone(),
					freezer: admin.clone(),
					supply,
					deposit,
					min_balance,
					is_sufficient: false,
					accounts: 1,
					sufficients: 0,
					approvals: 0,
					is_frozen: false,
				};
				Asset::<T, I>::insert(token_id, details);

				let new_deposit = T::MetadataDepositPerByte::get()
					.saturating_mul(((name.len() + symbol.len()) as u32).into())
					.saturating_add(T::MetadataDepositBase::get());
				T::Currency::reserve(&owner, new_deposit)?;

				let metadata = AssetMetadata {
					deposit: new_deposit,
					name: bounded_name,
					symbol: bounded_symbol,
					decimals,
					is_frozen: false,
				};
				// if_std!(
				//     println!("Update event {:?}", &metadata);
				// );
				Metadata::<T, I>::insert(token_id, metadata.clone());
				Self::deposit_event(Event::CreateMinted(
					token_id,
					owner,
					admin,
					beneficiary.clone(),
					metadata.clone(),
				));

				Ok(())
			})?;

			Ok(())
		}

		/// Issue a new class of fungible assets from a privileged origin.
		///
		/// This new asset class has no assets initially.
		///
		/// The origin must conform to `ForceOrigin`.
		///
		/// Unlike `create`, no funds are reserved.
		///
		/// - `id`: The identifier of the new asset. This must not be currently in use to identify
		/// an existing asset.
		/// - `owner`: The owner of this class of assets. The owner has full superuser permissions.
		/// over this asset, but may later change and configure the permissions using
		/// `transfer_ownership` and `set_team`.
		/// - `max_zombies`: The total number of accounts which may hold assets in this class yet
		/// have no existential deposit.
		/// - `min_balance`: The minimum balance of this new asset that any single account must
		/// have. If an accounts balance is reduced below this, then it collapses to zero.
		///
		/// Emits `ForceCreated` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::force_create())]
		pub fn force_create(
			origin: OriginFor<T>,
			id: T::AssetId,
			owner: <T::Lookup as StaticLookup>::Source,
			is_sufficient: bool,
			#[pallet::compact] min_balance: T::Balance,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let owner = T::Lookup::lookup(owner)?;

			ensure!(!Asset::<T, I>::contains_key(id), Error::<T, I>::InUse);
			ensure!(!min_balance.is_zero(), Error::<T, I>::MinBalanceZero);

			Asset::<T, I>::insert(
				id,
				AssetDetails {
					owner: owner.clone(),
					issuer: owner.clone(),
					admin: owner.clone(),
					freezer: owner.clone(),
					supply: Zero::zero(),
					deposit: Zero::zero(),
					min_balance,
					is_sufficient,
					accounts: 0,
					sufficients: 0,
					approvals: 0,
					is_frozen: false,
				},
			);
			Self::deposit_event(Event::ForceCreated(id, owner));
			Ok(())
		}

		/// Destroy a class of fungible assets.
		///
		/// The origin must conform to `ForceOrigin` or must be Signed and the sender must be the
		/// owner of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be destroyed. This must identity an existing
		/// asset.
		///
		/// Emits `Destroyed` event when successful.
		///
		/// Weight: `O(c + p +a)` where:
		/// - `c = (witness.accounts - witness.sufficients)`
		/// - `s = witness.sufficients`
		/// - `a = witness.approvals`
		#[pallet::weight(T::WeightInfo::destroy(
                witness.accounts.saturating_sub(witness.sufficients),
                witness.sufficients,
                // witness.approvals,
        ))]
		pub fn destroy(
			origin: OriginFor<T>,
			id: T::AssetId,
			witness: DestroyWitness,
		) -> DispatchResult {
			let maybe_check_owner = match T::ForceOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			Asset::<T, I>::try_mutate_exists(id, |maybe_details| {
				let mut details = maybe_details.take().ok_or(Error::<T, I>::Unknown)?;
				if let Some(check_owner) = maybe_check_owner {
					ensure!(details.owner == check_owner, Error::<T, I>::NoPermission);
				}
				ensure!(details.accounts == witness.accounts, Error::<T, I>::BadWitness);
				ensure!(details.sufficients == witness.sufficients, Error::<T, I>::BadWitness);
				ensure!(details.approvals == witness.approvals, Error::<T, I>::BadWitness);

				for (who, v) in Account::<T, I>::drain_prefix(id) {
					Self::dead_account(id, &who, &mut details, v.sufficient);
				}
				debug_assert_eq!(details.accounts, 0);
				debug_assert_eq!(details.sufficients, 0);

				let metadata = Metadata::<T, I>::take(&id);
				//let identity = <IdentityOf<T, I>>::take(&id).ok_or(Error::<T, I>::Unknown)?;
				let identity_deposit = match <IdentityOf<T, I>>::take(&id) {
					Some(identity) => identity.total_deposit(),
					None => Zero::zero(),
				};
				let deposit = details.deposit + metadata.deposit + identity_deposit;

				T::Currency::unreserve(
					&details.owner,
					deposit, /* details
					          *     .deposit
					          *     .saturating_add(metadata.deposit)
					          *     .saturating_add(identity.total_deposit()), */
				);

				for ((owner, _), approval) in Approvals::<T, I>::drain_prefix((&id,)) {
					T::Currency::unreserve(&owner, approval.deposit);
				}
				Self::deposit_event(Event::Destroyed(id));

				// NOTE: could use postinfo to reflect the actual number of
				// accounts/sufficient/approvals
				Ok(())
			})
		}

		/// Mint assets of a particular class.
		///
		/// The origin must be Signed and the sender must be thi Issuer of the asset `id`.
		///
		/// `id`: The identifier of the asset to have some amount minted.
		/// `beneficiary`: The account to be credited with the minted assets.
		/// `amount`: The amount of the asset to be minted.
		///
		/// Emits `Destroyed` event when successful.
		///
		/// Weight: `O(1)`
		/// Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`.
		#[pallet::weight(T::WeightInfo::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			id: T::AssetId,
			beneficiary: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;

			Self::do_mint(id, &beneficiary, amount, Some(origin))?;
			Ok(())
		}

		/// Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
		///
		/// - `id`: The identifier of the asset to have some amount burned.
		/// - `who`: The account to be debited from.
		/// - `amount`: The maximum amount by which `who`'s balance should be reduced.
		///
		/// Emits `Burned` with the actual amount burned. If this takes the balance to below the
		/// minimum for the asset, then the amount burned is increased to take it to zero.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] id: T::AssetId,
			who: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let who = T::Lookup::lookup(who)?;

			let f = DebitFlags { keep_alive: false, best_effort: true };
			let _ = Self::do_burn(id, &who, amount, Some(origin), f)?;

			Ok(())
		}

		/// Move some assets from the sender account to another.
		///
		/// Origin must be Signed.
		///
		/// - `id`: The identifier of the asset to have some amount transferred.
		/// - `target`: The account to be credited.
		/// - `amount`: The amount by which the sender's balance of assets should be reduced and
		/// `target`'s balance increase. The amount actually transferred may be slightly greather
		/// in the case that the transfer would otherwise take the sender balance above zero but
		/// below the minimum balance. Must be greather than zero.
		///
		/// Emits `Transferred` with the actual amount transferred. If this takes the source
		/// balance to below the minimum for the asset, then the amount transferred is increased to
		/// take it to zero.
		///
		/// Weight: `O(1)`
		/// Modes: Pre-existence of `target`; Post-existence of sender; Prior & post zombie-status
		/// of sender; Account pre-existence of `target`.
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: T::AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: T::Balance,
		) -> DispatchResult {
			let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
			let origin = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(target)?;
			Self::do_transfer(id, &origin, &dest, amount, ExistenceRequirement::AllowDeath, f)?;
			Self::deposit_event(Event::Transferred(id, origin, dest, amount));
			Ok(())
		}

		/// Disallow further ungrivileged transfers from an account
		///
		/// Origin must be Signed and the sender should be the Freezer of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		/// - `who`: The account to be frozen.
		///
		/// Emits `Frozen`.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::freeze())]
		pub fn freeze(
			origin: OriginFor<T>,
			id: T::AssetId,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(&origin == &d.freezer, Error::<T, I>::NoPermission);
			let who = T::Lookup::lookup(who)?;
			ensure!(Account::<T, I>::contains_key(id, &who), Error::<T, I>::BalanceZero);
			Account::<T, I>::mutate(id, &who, |a| a.is_frozen = true);
			Self::deposit_event(Event::<T, I>::Frozen(id, who));
			Ok(())
		}

		/// Allow unprivileged transfers from an account again.
		///
		/// Origin must be Signed and the sender should be the Admin of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		/// - `who`: The account to be unfrozen.
		///
		/// Emits `Thawed`
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::thaw())]
		pub fn thaw(
			origin: OriginFor<T>,
			id: T::AssetId,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			let details = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(&origin == &details.admin, Error::<T, I>::NoPermission);
			let who = T::Lookup::lookup(who)?;

			ensure!(Account::<T, I>::contains_key(id, &who), Error::<T, I>::BalanceZero);

			Account::<T, I>::mutate(id, &who, |a| a.is_frozen = false);

			Self::deposit_event(Event::<T, I>::Thawed(id, who));
			Ok(())
		}

		/// Disallow further ungrivileged transfers for the asset class.
		///
		/// Origin must be Signed and the sender should be the Freezer of the asset `id`.
		///
		/// - `id`: The identifier of the asset to be frozen.
		///
		/// Emits `Frozen`.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::freeze_asset())]
		pub fn freeze_asset(origin: OriginFor<T>, id: T::AssetId) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let d = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(&origin == &d.freezer, Error::<T, I>::NoPermission);

				d.is_frozen = true;

				Self::deposit_event(Event::<T, I>::AssetFrozen(id));
				Ok(())
			})
		}

		/// Allow unprivileged transfers for the asset again.
		///
		/// Origin must be Signed and the sender should be the Admin of the asset `id`
		///
		/// - `id`: The identifier of the asset to be frozen.
		///
		/// Emits `Thawed`.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::thaw_asset())]
		pub fn thaw_asset(origin: OriginFor<T>, id: T::AssetId) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			Asset::<T, I>::try_mutate(id, |maybe_details| {
				let d = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				ensure!(&origin == &d.admin, Error::<T, I>::NoPermission);

				d.is_frozen = false;
				Self::deposit_event(Event::<T, I>::AssetThawed(id));
				Ok(())
			})
		}
		/// Verify asset from a privileged origin.
		///
		/// The origin must conform to `ForceOrigin`.
		///
		/// - `id`: The identifier of the asset to verify.
		///
		/// Emits `AssetVerified`
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::verify_asset())]
		pub fn verify_asset(origin: OriginFor<T>, id: T::AssetId) -> DispatchResult {
			//let origin = ensure_signed(origin)?;
			T::ForceOrigin::ensure_origin(origin)?;
			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(!&d.is_frozen, Error::<T, I>::Frozen);

			IdentityOf::<T, I>::try_mutate(id, |maybe_details| {
				let identity = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
				identity.is_verifiable = true;
				Self::deposit_event(Event::<T, I>::AssetVerified(id));
				Ok(())
			})
		}

		#[pallet::weight(0)]
		pub fn set_blacklist(
			origin: OriginFor<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			AssetsBlacklist::<T, I>::mutate(|assets_blacklist| {
				assets_blacklist.insert((name.clone(), symbol.clone()));
				Self::deposit_event(Event::BlacklistSet(name.clone(), symbol.clone()));
			});

			Ok(())
		}

		/// Set the metadata for an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// Funds of sender are reserved according to the formula:
		/// `MetadataDepositBase + MetadataDepositPerByte * (name.len + symbol)` taking into
		/// account any already reserved funds.
		///
		/// - `id`: The identifier of the asset to update.
		/// - `name`: The user friendly name of this asset. Limited in length by `StringLimit`.
		/// - `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`.
		/// - `decimals`: The number of decimals this asset uses to represent on unit.
		///
		/// Emits `MetadataSet`
		///
		/// Weight: `O(1)`

		#[pallet::weight(T::WeightInfo::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			id: T::AssetId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			ensure!(
				!AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone())),
				Error::<T, I>::AssetBlacklist
			);

			let bounded_name: BoundedVec<u8, T::StringLimit> =
				name.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;
			let bounded_symbol: BoundedVec<u8, T::StringLimit> =
				symbol.clone().try_into().map_err(|_| Error::<T, I>::BadMetadata)?;

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(&origin == &d.owner, Error::<T, I>::NoPermission);

			Metadata::<T, I>::try_mutate_exists(id, |metadata| {
				ensure!(
					metadata.as_ref().map_or(true, |m| !m.is_frozen),
					Error::<T, I>::NoPermission
				);

				let old_deposit = metadata.take().map_or(Zero::zero(), |m| m.deposit);
				let new_deposit = T::MetadataDepositPerByte::get()
					.saturating_mul(((name.len() + symbol.len()) as u32).into())
					.saturating_add(T::MetadataDepositBase::get());

				if new_deposit > old_deposit {
					T::Currency::reserve(&origin, new_deposit - old_deposit)?;
				} else {
					T::Currency::unreserve(&origin, old_deposit - new_deposit);
				}
				*metadata = Some(AssetMetadata {
					deposit: new_deposit,
					name: bounded_name,
					symbol: bounded_symbol,
					decimals,
					is_frozen: false,
				});

				Self::deposit_event(Event::MetadataSet(id, name, symbol, decimals, false));
				Ok(())
			})
		}

		/// Clear the metadata for an asset.
		///
		/// Origin must be Signed and the sender should be the Owner of the asset `id`.
		///
		/// Any deposit is freed for the asset owner.
		///
		/// - `id`: The identifier of the asset to clear.
		///
		/// Emits `MetadataCleared`.
		///
		/// Weight: `O(1)`
		#[pallet::weight(T::WeightInfo::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, id: T::AssetId) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(&origin == &d.owner, Error::<T, I>::NoPermission);

			Metadata::<T, I>::try_mutate_exists(id, |metadata| {
				let deposit = metadata.take().ok_or(Error::<T, I>::Unknown)?.deposit;
				T::Currency::unreserve(&d.owner, deposit);
				Self::deposit_event(Event::MetadataCleared(id));
				Ok(())
			})
		}

		/// Set an account's identity information and reserve the appropriate deposit.
		///
		/// If the asset already has identity information, the deposit is taken as part payment
		/// for new deposit.
		///
		/// The dispatch origin for this call must be _Signed_
		///
		/// - `info`: The identity information.
		/// Emits `IdentitySet` if successful.
		///
		/// # <weight>
		/// - `O(X+X' + R)`
		/// - where `X` additional-field-count (deposit-bounded and code-bounded)
		/// - where `R` judgements-count (registrar-count-bounded)
		/// - One balance reserve operation.
		/// - One storage mutation (codec-read `O(X' + R)`, codec-write `O(X + R)`).
		/// - One event.
		/// #</weight>
		#[pallet::weight(T::WeightInfo::set_identity(
                //T::MaxRegistrars::get().into(), // R
                T::MaxAdditionalFields::get().into(), // X
        ))]
		pub fn set_identity(
			origin: OriginFor<T>,
			id: T::AssetId,
			info: AssetIdentity,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;
			let extra_fields = info.additional.len() as u32;
			ensure!(extra_fields <= T::MaxAdditionalFields::get(), Error::<T, I>::TooManyFields);
			let d = Asset::<T, I>::get(id).ok_or(Error::<T, I>::Unknown)?;
			ensure!(&origin == &d.owner, Error::<T, I>::NoPermission);
			ensure!(!&d.is_frozen, Error::<T, I>::Frozen);

			let fd = <BalanceOf<T, I>>::from(extra_fields) * T::FieldDeposit::get();
			let mut identity = match <IdentityOf<T, I>>::get(&id) {
				Some(mut identity) => {
					identity.info = info;
					identity.is_verifiable = false;
					identity
				}
				None => Registration { info, is_verifiable: false, deposit: Zero::zero() },
			};
			let old_deposit = identity.deposit;
			identity.deposit = T::BasicDeposit::get() + fd;
			if old_deposit > identity.deposit {
				let err_amount = T::Currency::unreserve(&origin, old_deposit - identity.deposit);
				debug_assert!(err_amount.is_zero());
			}

			<IdentityOf<T, I>>::insert(&id, identity);
			Self::deposit_event(Event::IdentitySet(id));
			Ok(Some(T::WeightInfo::set_identity(extra_fields)).into())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub(crate) fn set_genesis(who: &T::AccountId, amount: T::Balance) {
		Self::do_set_genesis(who, amount).unwrap();
	}

	pub(crate) fn try_mutate_account<R, E>(
		who: &T::AccountId,
		currency_id: T::AssetId,
		f: impl FnOnce(&mut AssetBalance<T::Balance, T::Extra>, bool) -> sp_std::result::Result<R, E>,
	) -> sp_std::result::Result<R, E> {
		Account::<T, I>::try_mutate_exists(currency_id, who, |maybe_account| {
			let existed = maybe_account.is_some();
			let mut account = maybe_account.take().unwrap_or_default();
			f(&mut account, existed).map(move |result| {
				let maybe_endowed = if !existed { Some(account.free) } else { None };
				let mut maybe_dust: Option<T::Balance> = None;
				let total = account.total();
				*maybe_account = if total.is_zero() {
					None
				} else {
					// if non_zero totalis below existential deposit, should handle the dust.
					if total < T::ExistentialDeposits::get(&currency_id) {
						maybe_dust = Some(total);
					}

					Some(account)
				};

				(maybe_endowed, existed, maybe_account.is_some(), maybe_dust, result)
			})
		})
		.map(|(maybe_endowed, existed, exists, _maybe_dust, result)| {
			if existed && !exists {
				// If existed before, decrease account provider.
				// Ignore the result, because if it failed means that these’s remain consumers,
				// and the account storage in frame_system shouldn't be repeaded.
				let _ = frame_system::Pallet::<T>::dec_providers(who);
			} else if !existed && exists {
				// if new, increase account provider
				frame_system::Pallet::<T>::inc_providers(who);
			}

			if let Some(endowed) = maybe_endowed {
				Self::deposit_event(Event::Endowed(currency_id, who.clone(), endowed));
			}

			// if let Some(dust_amount) = handle_dust {
			//     // `OnDust` maybe get/set storage `Accounts` of `who`, trigger handler here
			//     // to avoid some unexpected errors.
			//     T::OnDust::on_dust(who, currency_id, dust_amount);
			//     Self::deposit_event(Event::DustLost(who.clone(), currency_id, dust_amount));
			// }

			result
		})
	}

	pub(crate) fn mutate_account<R>(
		who: &T::AccountId,
		currency_id: T::AssetId,
		f: impl FnOnce(&mut AssetBalance<T::Balance, T::Extra>, bool) -> R,
	) -> R {
		Self::try_mutate_account(who, currency_id, |account, existed| -> Result<R, Infallible> {
			Ok(f(account, existed))
		})
		.expect("Error is infallible; qed")
	}

	/// Set free balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance, and the caller is expected to do it.

	pub(crate) fn set_free_balance(
		currency_id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
	) {
		Self::mutate_account(who, currency_id, |account, _| {
			account.free = amount;
		});
	}

	pub(crate) fn set_balance(
		currency_id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Asset::<T, I>::try_mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;

			Self::mutate_account(who, currency_id, |account, _| -> DispatchResult {
				account.free = amount;
				if account.free.is_zero() {
					account.sufficient = Self::new_account(&who, details)?;
				}
				Ok(())
			})?;
			Ok(())
		})?;
		Ok(())
	}

	/// Set reserved balance of `who` to a new value
	///
	/// Note this will not maintain total issuance, and the caller is
	/// expected to do it.
	pub(crate) fn set_reserved_balance(
		currency_id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
	) {
		Self::mutate_account(who, currency_id, |account, _| account.reserved = amount);
	}
}

impl<T: Config<I>, I: 'static> MultiCurrency<T::AccountId> for Pallet<T, I> {
	type CurrencyId = T::AssetId;
	type Balance = T::Balance;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		T::ExistentialDeposits::get(&currency_id)
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		//<TotalIssuance<T, I>>::get(currency_id)
		Self::total_issuance(currency_id)
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::account(currency_id, who).total()
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::account(currency_id, who).free
	}

	// Ensure that an account can withdraw from their free balance given any
	// existing withdrawl restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.

	fn ensure_can_withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		let _new_balance = Self::free_balance(currency_id, who)
			.checked_sub(&amount)
			.ok_or(Error::<T, I>::BalanceLow)?;
		// ensure!(
		//     new_balance >= Self::account(who, currency_id).frozen(),
		//     Error::<T>::LiquidityRestrictions
		// );
		Ok(())
	}

	/// Transfer some free balance from `from` to `to`.
	/// Is a no-op if value to be transferred is zero or the `from` is the same as `to`.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
		// Cannot underflow because ensure_can_withdraw check
		// Self::try_mutate_asset(currency_id, from, to, amount, f).map(|_| ())

		// Self::do_transfer(currency_id, &from, &to, amount, None, f).map(|_| ())
		Self::do_transfer(currency_id, from, to, amount, ExistenceRequirement::AllowDeath, f)
	}

	/// Deposit some `amount` into the free balance of account `who`.
	///
	/// Is a no-op if the `amount` to be deposited is zero.
	fn deposit(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		Asset::<T, I>::mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
			details.supply =
				details.supply.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;

			Self::set_free_balance(currency_id, who, Self::free_balance(currency_id, who) + amount);
			Ok(())
		})
	}

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		Self::ensure_can_withdraw(currency_id, who, amount)?;
		Asset::<T, I>::mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
			details.supply = details.supply.checked_sub(&amount).expect("cannot withdraw");
			Ok(())
		})?;

		Self::set_free_balance(currency_id, who, Self::free_balance(currency_id, who) - amount);
		Ok(())
	}

	// Check if `value` amount of free balance can be slashed from `who`.
	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::free_balance(currency_id, who) >= value
	}

	/// Is a no-op if `value` to be slashed is zero.
	///
	/// NOTE: `slash()` prefers free balance, but assumes that reserve
	/// balance can be drawn from in extreme circumstances. `can_slash()`
	/// should be used prior to `slash()` to avoid having to draw from
	/// reserved funds, however we err on the side of punishment if things
	/// are inconsistent or `can_slash` wasn't used appropriately.

	fn slash(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Self::Balance {
		if amount.is_zero() {
			return amount;
		}
		let account = Self::account(currency_id, who);
		let free_slashed_amount = account.free.min(amount);

		// Cannot underflow because free_slashed_amount can never be greater tha amount
		let mut remaining_slash = amount - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			// Cannot underflow because free_slashed_amount can never be greater than account.free
			Self::set_free_balance(currency_id, who, account.free - free_slashed_amount);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.reserved.min(remaining_slash);
			// Cannot underflow due to above line
			remaining_slash -= reserved_slashed_amount;
			Self::set_reserved_balance(
				currency_id,
				who,
				account.reserved - reserved_slashed_amount,
			);
		}

		// Cannot underflow because the slashed value cannot be greater than total issuance

		Asset::<T, I>::mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
			details.supply -= amount - remaining_slash;
			Ok(())
		});
		remaining_slash
	}
}

impl<T: Config<I>, I: 'static> MultiCurrencyExtended<T::AccountId> for Pallet<T, I> {
	type Amount = T::Amount;

	fn update_balance(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		by_amount: Self::Amount,
	) -> DispatchResult {
		if by_amount.is_zero() {
			return Ok(());
		}

		// Ensure this doesn't overflow. There isn't any traits that exposes
		// `saturating_abs` so we need to do it manually.
		let by_amount_abs = if by_amount == Self::Amount::min_value() {
			Self::Amount::max_value()
		} else {
			by_amount.abs()
		};

		let by_balance = TryInto::<Self::Balance>::try_into(by_amount_abs)
			.map_err(|_| Error::<T, I>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(currency_id, who, by_balance)
		} else {
			Self::withdraw(currency_id, who, by_balance).map(|_| ())
		}
	}
}

impl<T: Config<I>, I: 'static> MultiLockableCurrency<T::AccountId> for Pallet<T, I> {
	type Moment = T::BlockNumber;

	// Set a lock on the balance of `who` under `currency_id`.
	// Is a no-op if lock amount is zero.
	fn set_lock(
		_lock_id: LockIdentifier,
		_currency_id: Self::CurrencyId,
		_who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		// let mut new_lock = Some(BalanceLock {
		//     id: lock_id,
		//     amount,
		// });
		// let mut locks = Self::locks(who, currency_id)
		//     .into_iter()
		//     .filter_map(|lock| {
		//         if lock.id == lock_id {
		//             new_lock.take()
		//         } else {
		//             Some(lock)
		//         }
		//     })
		//     .collect::<Vec<_>>();
		// if let Some(lock) = new_lock {
		//     locks.push(lock)
		// }
		// Self::update_locks(currency_id, who, &locks[..])
		Ok(())
	}

	// Extend a lock on the balance of `who` under `currency_id`.
	// Is a no-op if lock amount is zero
	fn extend_lock(
		_lock_id: LockIdentifier,
		_currency_id: Self::CurrencyId,
		_who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		// let mut new_lock = Some(BalanceLock {
		//     id: lock_id,
		//     amount,
		// });
		// let mut locks = Self::locks(who, currency_id)
		//     .into_iter()
		//     .filter_map(|lock| {
		//         if lock.id == lock_id {
		//             new_lock.take().map(|nl| BalanceLock {
		//                 id: lock.id,
		//                 amount: lock.amount.max(nl.amount),
		//             })
		//         } else {
		//             Some(lock)
		//         }
		//     })
		//     .collect::<Vec<_>>();
		// if let Some(lock) = new_lock {
		//     locks.push(lock)
		// }
		// Self::update_locks(currency_id, who, &locks[..])
		Ok(())
	}

	fn remove_lock(
		_lock_id: LockIdentifier,
		_currency_id: Self::CurrencyId,
		_who: &T::AccountId,
	) -> DispatchResult {
		// let mut locks = Self::locks(who, currency_id);
		// locks.retain(|lock| lock.id != lock_id);
		// let locks_vec = locks.to_vec();
		// Self::update_locks(currency_id, who, &locks_vec[..])
		Ok(())
	}
}

impl<T: Config<I>, I: 'static> MultiReservableCurrency<T::AccountId> for Pallet<T, I> {
	/// Check if `who` can reserve `value` from their free balance.
	///
	/// Always `true` if value to be reserved is zero.
	fn can_reserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::ensure_can_withdraw(currency_id, who, value).is_ok()
	}

	/// Slash from reserved balance, returning any amount that was unable to
	/// be slashed.
	///
	/// Is a no-op if the value to be slashed is zero.
	fn slash_reserved(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		if value.is_zero() {
			return value;
		}

		let reserved_balance = Self::reserved_balance(currency_id, who);
		let actual = reserved_balance.min(value);
		Self::set_reserved_balance(currency_id, who, reserved_balance - actual);
		Asset::<T, I>::mutate(currency_id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T, I>::Unknown)?;
			details.supply = details.supply.checked_sub(&actual).expect("cannot slash reserved");
			Ok(())
		});
		value - actual
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		Self::account(currency_id, who).reserved
	}

	/// Move `value` from the free balance from `who` to their reserved
	/// balance.
	///
	/// Is a no-op if value to be reserved is zero.
	fn reserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> DispatchResult {
		if value.is_zero() {
			return Ok(());
		}
		Self::ensure_can_withdraw(currency_id, who, value)?;

		let account = Self::account(currency_id, who);
		Self::set_free_balance(currency_id, who, account.free - value);
		// Cannot overflow becuase total issuance is using the same balance type and
		// this doesn't increase total issuance
		Self::set_reserved_balance(currency_id, who, account.reserved + value);
		Ok(())
	}

	/// Unreserve some funds, returning any amount that was unable to be
	/// unreserved.
	///
	/// Is a no-op if the value to be unreserved is zero.
	fn unreserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		if value.is_zero() {
			return value;
		}

		let account = Self::account(currency_id, who);
		let actual = account.reserved.min(value);
		Self::set_reserved_balance(currency_id, who, account.reserved - actual);
		Self::set_free_balance(currency_id, who, account.free + actual);

		value - actual
	}

	/// Move the reserved balance of one account into the balance of
	/// another, according to `status`.
	///
	/// Is a no-op if:
	/// - the value to be moved is zero; or
	/// - the `slashed` id equal to `beneficiary` and the `status` is `Reserved`.
	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> sp_std::result::Result<Self::Balance, DispatchError> {
		if value.is_zero() {
			return Ok(value);
		}

		if slashed == beneficiary {
			return match status {
				BalanceStatus::Free => Ok(Self::unreserve(currency_id, slashed, value)),
				BalanceStatus::Reserved => {
					Ok(value.saturating_sub(Self::reserved_balance(currency_id, slashed)))
				}
			};
		}

		let from_account = Self::account(currency_id, slashed);
		let to_account = Self::account(currency_id, beneficiary);
		let actual = from_account.reserved.min(value);
		match status {
			BalanceStatus::Free => {
				Self::set_free_balance(currency_id, beneficiary, to_account.free + actual);
			}
			BalanceStatus::Reserved => {
				Self::set_reserved_balance(currency_id, beneficiary, to_account.reserved + actual);
			}
		}
		Self::set_reserved_balance(currency_id, slashed, from_account.reserved - actual);
		Ok(value - actual)
	}
}

pub struct CurrencyAdapter<T, GetCurrencyId>(marker::PhantomData<(T, GetCurrencyId)>);

impl<T, GetCurrencyId> PalletCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::AssetId>,
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, GetCurrencyId>;
	type NegativeImbalance = NegativeImbalance<T, GetCurrencyId>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::total_balance(GetCurrencyId::get(), who)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_slash(GetCurrencyId::get(), who, value)
	}

	fn total_issuance() -> Self::Balance {
		Pallet::<T>::total_issuance(GetCurrencyId::get())
	}

	fn minimum_balance() -> Self::Balance {
		Pallet::<T>::minimum_balance(GetCurrencyId::get())
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		if amount.is_zero() {
			return PositiveImbalance::zero();
		}
		// <TotalIssuance<T>>::mutate(GetCurrencyId::get(), |issued| {
		//     *issued = issued.checked_sub(&amount).unwrap_or_else(|| {
		//         amount = *issued;
		//         Zero::zero()
		//     });
		// });

		Asset::<T>::mutate(GetCurrencyId::get(), |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			details.supply = details.supply.checked_sub(&amount).unwrap_or_else(|| {
				amount = details.supply;
				Zero::zero()
			});
			Ok(())
		});

		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		if amount.is_zero() {
			return NegativeImbalance::zero();
		}

		Asset::<T>::mutate(GetCurrencyId::get(), |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			details.supply = details.supply.checked_add(&amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - details.supply;
				Self::Balance::max_value()
			});
			Ok(())
		});

		NegativeImbalance::new(amount)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		_reasons: WithdrawReasons,
		_new_balance: Self::Balance,
	) -> DispatchResult {
		Pallet::<T>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(
		source: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		_existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		<Pallet<T> as MultiCurrency<T::AccountId>>::transfer(
			GetCurrencyId::get(),
			&source,
			&dest,
			value,
		)
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		if value.is_zero() {
			return (Self::NegativeImbalance::zero(), value);
		}

		let currency_id = GetCurrencyId::get();
		let account = Pallet::<T>::account(currency_id, who);
		let free_slashed_amount = account.free.min(value);
		let mut remaining_slash = value - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			Pallet::<T>::set_free_balance(currency_id, who, account.free - free_slashed_amount);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.reserved.min(remaining_slash);
			remaining_slash -= reserved_slashed_amount;
			Pallet::<T>::set_reserved_balance(
				currency_id,
				who,
				account.reserved - reserved_slashed_amount,
			);
			(
				Self::NegativeImbalance::new(free_slashed_amount + reserved_slashed_amount),
				remaining_slash,
			)
		} else {
			(Self::NegativeImbalance::new(value), remaining_slash)
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> sp_std::result::Result<Self::PositiveImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::PositiveImbalance::zero());
		}
		let currency_id = GetCurrencyId::get();
		let new_total = Pallet::<T>::free_balance(currency_id, who)
			.checked_add(&value)
			.ok_or(ArithmeticError::Overflow)?;
		Pallet::<T>::set_free_balance(currency_id, who, new_total);

		Ok(Self::PositiveImbalance::new(value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		Self::deposit_into_existing(who, value).unwrap_or_else(|_| Self::PositiveImbalance::zero())
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		_reasons: WithdrawReasons,
		_liveness: ExistenceRequirement,
	) -> sp_std::result::Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::NegativeImbalance::zero());
		}
		let currency_id = GetCurrencyId::get();
		Pallet::<T>::ensure_can_withdraw(currency_id, who, value)?;
		Pallet::<T>::set_free_balance(
			currency_id,
			who,
			Pallet::<T>::free_balance(currency_id, who) - value,
		);

		Ok(Self::NegativeImbalance::new(value))
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		value: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		let currency_id = GetCurrencyId::get();
		Pallet::<T>::try_mutate_account(
			who,
			currency_id,
			|account,
			 existed|
			 -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, ()> {
				// If we're attempting to set an existing account to less than ED, then
				// bypass the entire operation. It's a no-op if you follow it through, but
				// since this is an instance where we might account for a negative imbalance
				// (in the dust cleaner of set_account) before we account for its actual
				// equal and opposite cause (returned as an Imbalance), then in the
				// instance that there's no other accounts on the system at all, we might
				// underflow the issuance and our arithmetic will be off.
				let ed = T::ExistentialDeposits::get(&currency_id);
				ensure!(value.saturating_add(account.reserved) >= ed || existed, ());

				let imbalance = if account.free <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.free))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.free - value))
				};
				account.free = value;
				Ok(imbalance)
			},
		)
		.unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
	}
}

impl<T, GetCurrencyId> PalletReservableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::AssetId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(
		who: &T::AccountId,
		value: Self::Balance,
	) -> (Self::NegativeImbalance, Self::Balance) {
		let actual = Pallet::<T>::slash_reserved(GetCurrencyId::get(), who, value);
		(Self::NegativeImbalance::zero(), actual)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		Pallet::<T>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		Pallet::<T>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: Status,
	) -> sp_std::result::Result<Self::Balance, DispatchError> {
		Pallet::<T>::repatriate_reserved(GetCurrencyId::get(), slashed, beneficiary, value, status)
	}
}

impl<T, GetCurrencyId> PalletLockableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::AssetId>,
{
	type Moment = T::BlockNumber;
	type MaxLocks = ();

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
		_reasons: WithdrawReasons,
	) {
		let _ = Pallet::<T>::set_lock(id, GetCurrencyId::get(), who, amount);
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
		_reasons: WithdrawReasons,
	) {
		let _ = Pallet::<T>::extend_lock(id, GetCurrencyId::get(), who, amount);
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let _ = Pallet::<T>::remove_lock(id, GetCurrencyId::get(), who);
	}
}

impl<T: Config> TransferAll<T::AccountId> for Pallet<T> {
	#[transactional]
	fn transfer_all(_source: &T::AccountId, _dest: &T::AccountId) -> DispatchResult {
		// Account::<T>::iter_prefix(source).try_for_each(
		//     |(currency_id, account_data)| -> DispatchResult {
		//         <Self as MultiCurrency<T::AccountId>>::transfer(
		//             currency_id,
		//             source,
		//             dest,
		//             account_data.free,
		//         )
		//     },
		// )
		Ok(())
	}
}
