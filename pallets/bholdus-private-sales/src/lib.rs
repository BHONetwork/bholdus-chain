#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::codec::{Decode, Encode};
use frame_support::traits::LockIdentifier;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type BalanceUnlockPercent = u8;
pub type RoundId = u32;

/// Lock Setting for a Private Sale Round
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct BalanceLockSetting<Balance, BlockNumber> {
    /// List of percents that is unlocked at each stage
    pub unlock_percents: Vec<BalanceUnlockPercent>,
    /// List of durations (number of blocks) funds will be locked at each stage
    pub lock_durations: Vec<BlockNumber>,
    /// List of lock identifiers for each stage
    pub lock_identifiers: Vec<LockIdentifier>,
    /// Minimum balance to transfer
    pub min_transfer_amount: Option<Balance>,
    /// Maximum balance to transfer
    pub max_transfer_amount: Option<Balance>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Dispatchable},
        pallet_prelude::*,
        traits::{
            schedule::{DispatchTime, Named as NamedScheduler},
            Currency, ExistenceRequirement, LockableCurrency, WithdrawReasons,
        },
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::StaticLookup, Percent};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency mechanism
        type Currency: Currency<Self::AccountId> + LockableCurrency<Self::AccountId>;

        /// The origin which may call force operations
        type ForceOrigin: EnsureOrigin<Self::Origin>;

        /// The aggregated call type
        type Call: Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

        /// Overarching type of all pallets origins.
        type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

        /// Scheduler
        type Scheduler: NamedScheduler<
            Self::BlockNumber,
            <Self as Config>::Call,
            Self::PalletsOrigin,
        >;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Lock settings storage
    #[pallet::storage]
    pub type BalanceLockSettings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        RoundId,
        BalanceLockSetting<BalanceOf<T>, BlockNumberFor<T>>,
    >;

    /// Participating users tracking storage
    /// Return true if a user already participated in a private sales round
    /// Otherwise, return false
    #[pallet::storage]
    pub type ParticipatingUsersTracking<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, RoundId, Blake2_128Concat, T::AccountId, bool>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted when successfully create a private sale round. [round_id]
        RoundCreated(RoundId),
        /// Emitted when successfully transfer the funds. [round_id, from, to, amount]
        FundsTransferred(RoundId, T::AccountId, T::AccountId, BalanceOf<T>),
        /// Emitted when successufully lock the funds by lock setting [round_id, who, lock_identifier]
        FundsLocked(RoundId, T::AccountId, LockIdentifier),
        /// Emitted when successfully unlock the funds [who, lock_identifier]
        FundsUnlocked(T::AccountId, LockIdentifier),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Total balance unlock percent is not equal to 100
        InvalidTotalUnlockPercent,
        /// Unlock percent, Lock duration, Lock identifiers length mismatch
        LockSettingMismatch,
        /// Min amount required
        MinAmountRequired,
        /// Max amount required
        MaxAmountRequired,
        /// Lock Setting not found
        LockSettingNotFound,
        /// User already participated the private sale round
        UserAlreadyParticipated,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create Unlock Setting Item.
        ///
        /// This call is for Root user only.
        #[pallet::weight(10_000)]
        pub fn force_create_round(
            origin: OriginFor<T>,
            round_id: RoundId,
            balance_unlock_percents: Vec<BalanceUnlockPercent>,
            balance_lock_durations: Vec<BlockNumberFor<T>>,
            lock_identifiers: Vec<LockIdentifier>,
            min_transfer_amount: Option<BalanceOf<T>>,
            max_transfer_amount: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            let total_percent = balance_unlock_percents.iter().fold(0, |acc, x| acc + x);

            ensure!(total_percent == 100, Error::<T>::InvalidTotalUnlockPercent);

            ensure!(
                balance_unlock_percents.len() == balance_lock_durations.len(),
                Error::<T>::LockSettingMismatch
            );

            ensure!(
                balance_unlock_percents.len() == lock_identifiers.len(),
                Error::<T>::LockSettingMismatch
            );

            let balance_release_setting = BalanceLockSetting {
                unlock_percents: balance_unlock_percents,
                lock_durations: balance_lock_durations,
                min_transfer_amount: min_transfer_amount,
                max_transfer_amount: max_transfer_amount,
                lock_identifiers: lock_identifiers,
            };
            <BalanceLockSettings<T>>::insert(round_id, balance_release_setting);

            Self::deposit_event(Event::RoundCreated(round_id));

            Ok(().into())
        }

        /// Transfer funds to a user participating in Private Sale.
        ///
        /// Funds are also locked by specified Lock Setting.
        ///
        /// Because Lock Identifiers for each user in a private sales round are constants,
        /// Users can only participate a private sales round once.
        #[pallet::weight(10_000)]
        pub fn force_transfer_and_lock(
            origin: OriginFor<T>,
            from: <T::Lookup as StaticLookup>::Source,
            to: <T::Lookup as StaticLookup>::Source,
            amount: BalanceOf<T>,
            round_id: RoundId,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            let from_id = T::Lookup::lookup(from.clone())?;
            let to_id = T::Lookup::lookup(to.clone())?;

            let already_participated =
                <ParticipatingUsersTracking<T>>::contains_key(round_id, to_id.clone());
            ensure!(
                already_participated == false,
                Error::<T>::UserAlreadyParticipated
            );

            let lock_setting = match <BalanceLockSettings<T>>::get(round_id) {
                Some(ls) => ls,
                None => {
                    return Err((Error::<T>::LockSettingNotFound).into());
                }
            };

            if let Some(min_transfer_amount) = lock_setting.min_transfer_amount {
                ensure!(amount >= min_transfer_amount, Error::<T>::MinAmountRequired);
            }

            if let Some(max_transfer_amount) = lock_setting.max_transfer_amount {
                ensure!(amount <= max_transfer_amount, Error::<T>::MaxAmountRequired);
            }

            T::Currency::transfer(&from_id, &to_id, amount, ExistenceRequirement::KeepAlive)?;

            // Lock funds and scheduling to unlock
            for (i, unlock_percent) in lock_setting.unlock_percents.iter().enumerate() {
                // Lock the funds
                T::Currency::set_lock(
                    lock_setting.lock_identifiers[i],
                    &to_id,
                    Percent::from_percent(*unlock_percent) * amount,
                    WithdrawReasons::all(),
                );

                // Schedule to unlock
                let block_number = lock_setting.lock_durations[i];

                let schedule_result = T::Scheduler::schedule_named(
                    (round_id, lock_setting.lock_identifiers[i], to_id.clone()).encode(),
                    DispatchTime::After(block_number),
                    None,
                    84,
                    frame_system::RawOrigin::Root.into(),
                    Call::force_unlock(to.clone(), lock_setting.lock_identifiers[i]).into(),
                );

                // Since scheduling id is constant for a user participating in a private sale round,
                // scheduling may fail if user tries to participate more than once.
                // This case should not happen since we have a user tracking check above.
                // However, we still put this handling error code here since compiler is giving a warning.
                if schedule_result.is_err() {
                    frame_support::print("LOGIC ERROR: bholdus-private-sales/force_transfer_and_lock scheduling unlock failed");
                    frame_support::print("bholdus-private-sales/force_transfer_and_lock attempts to retry scheduling unlock");
                }
            }

            <ParticipatingUsersTracking<T>>::insert(round_id, to_id.clone(), true);

            Self::deposit_event(Event::FundsTransferred(
                round_id,
                from_id,
                to_id.clone(),
                amount,
            ));

            lock_setting.lock_identifiers.iter().for_each(|li| {
                Self::deposit_event(Event::FundsLocked(round_id, to_id.clone(), *li));
            });

            Ok(().into())
        }

        ///
        #[pallet::weight(10_000)]
        pub fn force_unlock(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            lock_identifier: LockIdentifier,
        ) -> DispatchResultWithPostInfo {
            T::ForceOrigin::ensure_origin(origin)?;

            let who_id = T::Lookup::lookup(who)?;

            T::Currency::remove_lock(lock_identifier, &who_id);

            Self::deposit_event(Event::FundsUnlocked(who_id.clone(), lock_identifier));

            Ok(().into())
        }
    }
}
