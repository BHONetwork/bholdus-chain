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

mod migrations;

pub mod weights;

use weights::WeightInfo;

type TransferId = u128;
type Bytes = Vec<u8>;
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
        /// Minimum amount to transfer. This should match `ExistentialDeposit` of `pallet_balance`
        #[pallet::constant]
        type MinimumDeposit: Get<BalanceOf<Self>>;
        /// Weight info
        type WeightInfo: weights::WeightInfo;
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    #[pallet::extra_constants]
    impl<T: Config> Pallet<T> {
        pub fn pallet_id() -> PalletId {
            PalletId(*b"xnatrans")
        }
        pub fn pallet_account_id() -> T::AccountId {
            Self::pallet_id().into_account()
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        service_fee: BalanceOf<T>,
        platform_fee: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            Self {
                service_fee: 0,
                platform_fee: 0,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            ServiceFee::<T>::put(self.service_fee.clone());
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            migrations::migrate::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            Ok(())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            Ok(())
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

    /// The service fee to charge users
    #[pallet::storage]
    #[pallet::getter(fn service_fee)]
    pub(super) type ServiceFee<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The platform fee to charge users
    #[pallet::storage]
    #[pallet::getter(fn platform_fee)]
    pub(super) type PlatformFee<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

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

    /// Indicating the bridge is frozen by admin
    #[pallet::storage]
    #[pallet::getter(fn is_frozen)]
    pub(super) type Frozen<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// User initiated a crosschain transfer successfully. [outbound_transfer_id, from, to, amount]
        OutboundTransferInitiated(TransferId, T::AccountId, Bytes, BalanceOf<T>),
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
        /// Minimum deposit required
        MinimumDepositRequired,
        /// Bridge is freezed
        Frozen,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Outbound
        /// User should submit this extrinsic to initiate a crosschain transfer.
        /// User will be charged with additional service fee.
        /// This service fee is rewarded to the relayer if the execution on the target chain succeeds.
        #[pallet::weight(T::WeightInfo::initiate_transfer(1))]
        #[transactional]
        pub fn initiate_transfer(
            origin: OriginFor<T>,
            to: Bytes,
            amount: BalanceOf<T>,
            target_chain: ChainId,
        ) -> DispatchResult {
            ensure!(!Self::is_frozen(), Error::<T>::Frozen);

            let who = ensure_signed(origin)?;

            // Only registered chains can receive the fund
            ensure!(
                RegisteredChains::<T>::get(target_chain),
                Error::<T>::MustBeRegisteredChain
            );

            ensure!(
                amount >= T::MinimumDeposit::get(),
                Error::<T>::MinimumDepositRequired
            );

            let fee = Self::service_fee()
                .checked_add(Self::platform_fee())
                .ok_or(ArithmeticError::Overflow)?;

            let total_charge = fee.checked_add(amount).ok_or(ArithmeticError::Overflow)?;

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
                    service_fee: fee,
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
        #[pallet::weight(T::WeightInfo::confirm_transfer(0))]
        #[transactional]
        pub fn confirm_transfer(origin: OriginFor<T>, transfer_id: TransferId) -> DispatchResult {
            ensure!(!Self::is_frozen(), Error::<T>::Frozen);

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
        #[pallet::weight(T::WeightInfo::release_tokens(0))]
        #[transactional]
        pub fn release_tokens(
            origin: OriginFor<T>,
            transfer_id: TransferId,
            from: Bytes,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(!Self::is_frozen(), Error::<T>::Frozen);

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

        /// Register relayer account responsible for relaying transfer between chains
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_register_relayer())]
        #[transactional]
        pub fn force_register_relayer(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredRelayers::<T>::insert(relayer, true);

            Ok(())
        }

        /// Unregister relayer account
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_unregister_relayer())]
        pub fn force_unregister_relayer(
            origin: OriginFor<T>,
            relayer: T::AccountId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredRelayers::<T>::insert(relayer, false);

            Ok(())
        }

        /// Register chain id that crosschain transfer supports
        /// Chain id will be pre-defined by Bholdus team
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_register_chain())]
        pub fn force_register_chain(origin: OriginFor<T>, chain: ChainId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredChains::<T>::insert(chain, true);

            Ok(())
        }

        /// Unregister chain id
        /// Chain id will be pre-defined by Bholdus team
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_unregister_chain())]
        pub fn force_unregister_chain(origin: OriginFor<T>, chain: ChainId) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            RegisteredChains::<T>::insert(chain, false);

            Ok(())
        }

        /// Set service fee that user will be charged when initiates a transfer
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_set_service_fee())]
        pub fn force_set_service_fee(
            origin: OriginFor<T>,
            service_fee: BalanceOf<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ServiceFee::<T>::put(service_fee);

            Ok(())
        }

        /// Set platform fee
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_set_platform_fee())]
        pub fn force_set_platform_fee(
            origin: OriginFor<T>,
            platform_fee: BalanceOf<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            PlatformFee::<T>::put(platform_fee);

            Ok(())
        }

        /// Withdraw tokens locked in this pallet to some account
        /// This operation is mainly used for any migration in the future
        /// Only `AdminOrigin` can access this operation
        #[pallet::weight(T::WeightInfo::force_withdraw())]
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

        /// Freeze the bridge
        #[pallet::weight(T::WeightInfo::force_freeze())]
        pub fn force_freeze(origin: OriginFor<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            Frozen::<T>::put(true);

            Ok(())
        }

        #[pallet::weight(T::WeightInfo::force_unfreeze())]
        pub fn force_unfreeze(origin: OriginFor<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin.clone())?;

            Frozen::<T>::put(false);

            Ok(())
        }
    }
}
