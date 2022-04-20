#![cfg_attr(not(feature = "std"), no_std)]

use common_primitives::Balance;
use frame_support::{traits::Currency, transactional, BoundedVec};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_runtime::traits::Zero;
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use types::*;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type UnixTime: UnixTime;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Native Currency trait
		type Currency: Currency<Self::AccountId, Balance = Balance>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: weights::WeightInfo;

		/// The maximum length of a name or symbol stored on-chain.
		type ContentLimit: Get<u32>;

		/// Admin Origin
		type AdminOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::type_value]
	pub fn AmountFreeTxDefault<T: Config>() -> u128 {
		100
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage

	#[pallet::storage]
	#[pallet::getter(fn memo)]
	pub type Memo<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ChainId,
		Blake2_128Concat,
		TxnHash,
		MemoInfo<T::AccountId, BoundedVec<u8, T::ContentLimit>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn memo_counter)]
	pub type MemoCounter<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn amount_free_tx)]
	pub type AmountFreeTx<T: Config> =
		StorageValue<Value = u128, QueryKind = ValueQuery, OnEmpty = AmountFreeTxDefault<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Some memo class was created \[chain_id, txn_hash, memo_info\]
		MemoCreated(ChainId, TxnHash, MemoInfo<T::AccountId, BoundedVec<u8, T::ContentLimit>>),
		/// Some memo class was updated \[chain_id, txn_hash, memo_info\]
		MemoUpdated(ChainId, TxnHash, MemoInfo<T::AccountId, BoundedVec<u8, T::ContentLimit>>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Memo is not existed
		NotExisted,
		/// The operator is not the operator of the memo
		NoPermission,
		/// Invalid memo info given.
		BadMemoInfo,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_: T::BlockNumber) -> Weight {
			0
		}

		fn on_finalize(_n: <T as frame_system::Config>::BlockNumber) {}

		fn on_runtime_upgrade() -> Weight {
			0
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn calc_actual_weight(sender: &Vec<u8>) -> Pays {
			let counter = Pallet::<T>::memo_counter(sender);
			if counter < Pallet::<T>::amount_free_tx() {
				Pays::No
			} else {
				Pays::Yes
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight((T::WeightInfo::create(content.len() as u32, txn_hash.len() as u32),
                            Pallet::<T>::calc_actual_weight(sender)))]
		#[transactional]
		pub fn create(
			origin: OriginFor<T>,
			chain_id: ChainId,
			txn_hash: TxnHash,
			content: Vec<u8>,
			sender: Vec<u8>,
			receiver: Vec<u8>,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let memo_info = Memo::<T>::get(&chain_id, &txn_hash);

			ensure!(memo_info.is_none(), Error::<T>::BadMemoInfo);

			let time_now = T::UnixTime::now().as_millis() as u64;

			let bounded_content: BoundedVec<u8, T::ContentLimit> =
				content.clone().try_into().map_err(|_| Error::<T>::BadMemoInfo)?;

			let counter: u128 = MemoCounter::<T>::get(&sender);

			MemoCounter::<T>::insert(&sender, counter.checked_add(1).unwrap_or_else(Zero::zero));

			let memo_info =
				MemoInfo { content: bounded_content, sender, receiver, operator, time: time_now };

			Memo::<T>::insert(&chain_id, &txn_hash, memo_info.clone());

			Self::deposit_event(Event::MemoCreated(chain_id, txn_hash, memo_info));
			Ok(())
		}

		// #[pallet::weight(T::WeightInfo::update(content.len() as u32))]
		// #[transactional]
		// pub fn update(
		//     origin: OriginFor<T>,
		//     chain_id: ChainId,
		//     txn_hash: TxnHash,
		//     content: Vec<u8>,
		// ) -> DispatchResult {
		//     let who = ensure_signed(origin)?;

		//     let memo_info = Memo::<T>::get(&chain_id, &txn_hash);

		//     ensure!(memo_info.is_some(), Error::<T>::NotExisted);

		//     let mut memo_info = memo_info.unwrap();

		//     ensure!(who == memo_info.operator, Error::<T>::NoPermission);

		//     memo_info.content = content;
		//     Memo::<T>::insert(&chain_id, &txn_hash, memo_info.clone());
		//     Self::deposit_event(Event::MemoUpdated(chain_id, txn_hash, memo_info));
		//     Ok(())
		// }

		#[pallet::weight(T::WeightInfo::set_amount_free_tx())]
		pub fn set_amount_free_tx(origin: OriginFor<T>, amount_free_tx: u128) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			<AmountFreeTx<T>>::put(amount_free_tx);

			Ok(())
		}
	}
}
