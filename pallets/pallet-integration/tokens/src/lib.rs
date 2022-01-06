#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use sp_runtime::traits::Saturating;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://docs.substrate.io/v3/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn get_balance)]
	pub(super) type BalanceToAccount<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		T::Balance,
		ValueQuery
		>;

    /// Token mint can emit two Event types.
    #[pallet::event]
    // #[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New token supply was minted.
		MintedNewSupply(T::AccountId),
		/// Tokens were successfully transferred between accounts. [from, to, value]
		Transferred(T::AccountId, T::AccountId, T::Balance),
	}

    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Errors inform users that something went wrong.
    #[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn transfer(origin: OriginFor<T>, dest: T::AccountId, tokenId: TokenId, amount: BalanceOf) -> DispatchResult {
            // ensure_signed(origin)?;
            // Do something with the value
            ContractEntry::<T>::put(amount);
            T::Currency::transfer(
                origin,
                dest,
                tokenId,
                amount
            )?;
            Self::deposit_event(Event::Transferred(origin, dest, amount));
            Ok(())
        }
	}
}

// impl<T: Config + bholdus_currencies::Config> Pallet<T> {
//     // #[pallet::weight(<T as pallet::Config>::WeightInfo::insert_number(*val))]
//     /// A storage extrinsic for demonstrating calls originating from a smart contract
//     /// * `val` - Some integer to be stored.
//     pub fn transfer(origin: OriginFor<T>, dest: T::AccountId, amount: T::Balance) -> DispatchResult {
//         // ensure_signed(origin)?;
//         // Do something with the value
//         bholdus_currencies::Pallet::<T>::transfer(
//             origin,
//             dest,
//             amount
//         )?;
//         Self::deposit_event(Event::CalledPalletFromContract(val));
//         Ok(())
//     }
// }
