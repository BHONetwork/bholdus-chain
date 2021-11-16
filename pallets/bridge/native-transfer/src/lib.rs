#![cfg_attr(not(feature = "std"), no_std)]

use bholdus_primitives::Balance;
use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::StorageVersion,
    traits::{Currency, ExistenceRequirement},
    transactional, Blake2_128Concat, PalletId, RuntimeDebug,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, ArithmeticError, FixedPointNumber, FixedU128};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type TransferId = u128;
type Bytes = Vec<u8>;
type FeeRate = (u32, u32);
type ChainId = u16;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct OutboundTransferInfo<AccountId, Balance, ChainId> {
    from: AccountId,
    to: Bytes,
    amount: Balance,
    target_chain: ChainId,
    service_fee: Balance,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OutboundTransferConfirmStatus {
    Failed,
    Successful,
}
impl Default for OutboundTransferConfirmStatus {
    fn default() -> Self {
        OutboundTransferConfirmStatus::Failed
    }
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum InboundTransferResultStatus {
    NotExist,
    Ok,
    Err(DispatchError),
}
impl Default for InboundTransferResultStatus {
    fn default() -> Self {
        InboundTransferResultStatus::NotExist
    }
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct InboundTransferResult {
    status: InboundTransferResultStatus,
}

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Native Currency trait
        type Currency: Currency<Self::AccountId, Balance = Balance>;
        /// Admin Origin
        type AdminOrigin: EnsureOrigin<Self::Origin>;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        pub fn pallet_id() -> PalletId {
            PalletId(*b"xnatrans")
        }
        pub fn pallet_account_id() -> T::AccountId {
            Self::pallet_id().into_account()
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::type_value]
    pub(super) fn DefaultNextOutboundTransferId() -> TransferId {
        0
    }
    #[pallet::type_value]
    pub(super) fn DefaultNextConfirmOutboundTransferId() -> TransferId {
        0
    }
    #[pallet::type_value]
    pub(super) fn DefaultServiceFeeRate() -> FeeRate {
        (3, 10_000)
    }
    #[pallet::type_value]
    pub(super) fn DefaultNextInboundTransferId() -> TransferId {
        0
    }

    /// Next outbound transfer id for the next transfer initiated by user.
    #[pallet::storage]
    #[pallet::getter(fn next_outbound_transfer_id)]
    pub(super) type NextOutboundTransferId<T> =
        StorageValue<_, TransferId, ValueQuery, DefaultNextOutboundTransferId>;

    /// The outbound transfer id waiting to be confirmed
    #[pallet::storage]
    #[pallet::getter(fn next_confirm_outbound_transfer_id)]
    pub(super) type NextConfirmOutboundTransferId<T> =
        StorageValue<_, TransferId, ValueQuery, DefaultNextConfirmOutboundTransferId>;

    /// Outbound transfers are stored here
    #[pallet::storage]
    #[pallet::getter(fn outbound_transfers)]
    pub(super) type OutboundTransfers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        TransferId,
        OutboundTransferInfo<T::AccountId, BalanceOf<T>, ChainId>,
        OptionQuery,
    >;

    /// The service fee rate to charge users
    #[pallet::storage]
    #[pallet::getter(fn service_fee_rate)]
    pub(super) type ServiceFeeRate<T> = StorageValue<_, FeeRate, ValueQuery, DefaultServiceFeeRate>;

    /// The inbound transfer id that should be received next
    #[pallet::storage]
    #[pallet::getter(fn next_inbound_transfer_id)]
    pub(super) type NextInboundTransferId<T> =
        StorageValue<_, TransferId, ValueQuery, DefaultNextInboundTransferId>;

    /// Registered Relayers.
    /// Only registered relayers can submit a transfer to release tokens to users
    #[pallet::storage]
    #[pallet::getter(fn registered_relayers)]
    pub(super) type RegisteredRelayers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    /// Registered Chains
    /// Only registered chains is supported for crosschain transfer
    #[pallet::storage]
    #[pallet::getter(fn registered_chains)]
    pub(super) type RegisteredChains<T: Config> =
        StorageMap<_, Blake2_128Concat, ChainId, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// User initiated a crosschain transfer successfully. [outbound_transfer_id, from, to, amount]
        OutboundTransferInitiated(TransferId, T::AccountId, Bytes, BalanceOf<T>),
        /// Inbound token release failed. [inbound_transfer_id, from, to, amount]
        InboundTokenReleaseFailed(TransferId, Bytes, T::AccountId, BalanceOf<T>),
        /// Inbound Token release succeeded. [inbound_transfer_id, from, to, amount]
        InboundTokenReleased(TransferId, Bytes, T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Origin must be registered relayers
        MustBeRegisteredRelayer,
        /// Only registered chains are supported
        MustBeRegisteredChain,
        /// Invalid Service Fee Rate
        InvalidServiceFeeRate,
        /// Outbound transfer is already confirmed or the transfer doesn't exists
        UnexpectedOutboundTransferConfirmation,
        /// All outbound transfers are confirmed
        AllOutboundTransfersConfirmed,
        /// Outbound transfer info not found,
        OutboundTransferNotFound,
        /// Inbound transfer received is already executed or the transfer doesn't exists
        UnexpectedInboundTransfer,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Outbound
        /// User should submit this extrinsic to initiate a crosschain transfer.
        /// User will be charged with additional service fee.
        /// This service fee is rewarded to the relayer if the execution on the target chain succeeds.
        #[pallet::weight(0)]
        #[transactional]
        pub fn initiate_transfer(
            origin: OriginFor<T>,
            to: Bytes,
            amount: BalanceOf<T>,
            target_chain: ChainId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered chains can receive the fund
            ensure!(
                RegisteredChains::<T>::get(target_chain),
                Error::<T>::MustBeRegisteredChain
            );

            let service_fee_rate = FixedU128::checked_from_rational(
                Self::service_fee_rate().0,
                Self::service_fee_rate().1,
            )
            .ok_or(Error::<T>::InvalidServiceFeeRate)?;

            let service_fee = service_fee_rate
                .checked_mul_int(amount)
                .ok_or(ArithmeticError::Overflow)?;

            let total_charge = service_fee
                .checked_add(amount)
                .ok_or(ArithmeticError::Overflow)?;

            // Lock user tokens
            T::Currency::transfer(
                &who,
                &Self::pallet_account_id(),
                total_charge,
                ExistenceRequirement::KeepAlive,
            )?;

            let transfer_id = Self::next_outbound_transfer_id();

            OutboundTransfers::<T>::insert(
                transfer_id,
                OutboundTransferInfo {
                    from: who.clone(),
                    to: to.clone(),
                    amount,
                    service_fee,
                    target_chain,
                },
            );

            let next_outbound_transfer_id = transfer_id
                .checked_add(1)
                .ok_or(ArithmeticError::Overflow)?;

            NextOutboundTransferId::<T>::put(next_outbound_transfer_id);

            Self::deposit_event(Event::OutboundTransferInitiated(
                transfer_id,
                who.clone(),
                to.clone(),
                amount,
            ));

            Ok(())
        }

        /// Outbound
        /// Relayer will call this function to confirm the transfer and claim their rewards
        #[pallet::weight(0)]
        #[transactional]
        pub fn confirm_transfer(origin: OriginFor<T>, transfer_id: TransferId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered relayers are allowed
            ensure!(
                Self::registered_relayers(who.clone()),
                Error::<T>::MustBeRegisteredRelayer
            );

            // Ignore if all outbound transfers are confirmed
            ensure!(
                Self::next_confirm_outbound_transfer_id() < Self::next_outbound_transfer_id(),
                Error::<T>::AllOutboundTransfersConfirmed
            );

            // Ignore if this transfer is already confirmed or doesn't exist
            ensure!(
                Self::next_confirm_outbound_transfer_id() == transfer_id,
                Error::<T>::UnexpectedOutboundTransferConfirmation
            );

            let outbound_transfer_info = Self::outbound_transfers(transfer_id)
                .ok_or(Error::<T>::OutboundTransferNotFound)?;

            T::Currency::transfer(
                &Self::pallet_account_id(),
                &who,
                outbound_transfer_info.service_fee,
                ExistenceRequirement::AllowDeath,
            )?;

            let next_confirm_outbound_transfer_id = Self::next_confirm_outbound_transfer_id()
                .checked_add(1)
                .ok_or(ArithmeticError::Overflow)?;

            NextConfirmOutboundTransferId::<T>::put(next_confirm_outbound_transfer_id);

            Ok(())
        }

        /// Inbound
        /// Relayer call this to release tokens to user corresponding to transfer sent from other chains.
        #[pallet::weight(0)]
        #[transactional]
        pub fn release_tokens(
            origin: OriginFor<T>,
            transfer_id: TransferId,
            from: Bytes,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Only registered relayers are allowed
            ensure!(
                Self::registered_relayers(who.clone()),
                Error::<T>::MustBeRegisteredRelayer
            );

            // Ignore if this transfer is already executed or the transfer_id doesn't exists
            ensure!(
                Self::next_inbound_transfer_id() == transfer_id,
                Error::<T>::UnexpectedInboundTransfer
            );

            T::Currency::transfer(
                &Self::pallet_account_id(),
                &to,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            let next_inbound_transfer_id = Self::next_inbound_transfer_id()
                .checked_add(1)
                .ok_or(ArithmeticError::Overflow)?;

            NextInboundTransferId::<T>::put(next_inbound_transfer_id);

            Self::deposit_event(Event::InboundTokenReleased(transfer_id, from, to, amount));

            Ok(())
        }

        #[pallet::weight(0)]
        #[transactional]
        pub fn force_register_relayer(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredRelayers::<T>::insert(relayer, true);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn force_unregister_relayer(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredRelayers::<T>::insert(relayer, false);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn force_register_chain(origin: OriginFor<T>, chain: ChainId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredChains::<T>::insert(chain, true);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn force_unregister_chain(origin: OriginFor<T>, chain: ChainId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredChains::<T>::insert(chain, false);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn force_set_service_fee(
            origin: OriginFor<T>,
            service_fee_rate: FeeRate,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            FixedU128::checked_from_rational(service_fee_rate.0, service_fee_rate.1)
                .ok_or(ArithmeticError::Overflow)?;

            ServiceFeeRate::<T>::put(service_fee_rate);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn force_withdraw(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            let locked_tokens = T::Currency::total_balance(&Self::pallet_account_id());
            T::Currency::transfer(
                &Self::pallet_account_id(),
                &to,
                locked_tokens,
                ExistenceRequirement::AllowDeath,
            )?;

            Ok(())
        }
    }
}
