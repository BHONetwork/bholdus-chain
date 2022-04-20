//! ### Functions
//!
//! - `create_clas` - Create NFT(non fungible token) class
//! - `transfer` - Transfer NFT to another account.
//! - `mint` - Mint NFT
//! - `burn` - Burn NFT
//! - `destroy_class` - Destroy NFT

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::unpper_case_acronyms)]

use enumflags2::BitFlags;
use frame_support::{
    log,
    pallet_prelude::*,
    require_transactional,
    traits::{
        Currency,
        ExistenceRequirement::{AllowDeath, KeepAlive},
        NamedReservableCurrency,
    },
    transactional, PalletId,
};

use scale_info::TypeInfo;

use frame_system::pallet_prelude::*;

use common_primitives::NFTBalance;
use bholdus_support::NFT;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
    DispatchResult, RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

pub type CID = Vec<u8>;
pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

pub type TokenIdOf<T> = <T as bholdus_support_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as bholdus_support_nft::Config>::ClassId;
pub type GroupIdOf<T> = <T as bholdus_support_nft::Config>::GroupId;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData {
    /// Class attributes
    pub attributes: Attributes,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {
    /// Token attributes
    pub attributes: Attributes,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + bholdus_support_nft::Config<ClassData = ClassData, TokenData = TokenData>
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The NFT's pallet id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The maximum quantity
        type MaxQuantity: Get<u32>;

        /// Maximum number of bytes in attributes
        #[pallet::constant]
        type MaxAttributesBytes: Get<u32>;

        /// Weight information for the extrinsics
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ClassId not found
        ClassIdNotFound,
        /// TokenId not found
        TokenIdNotFound,
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Quantity is invalid. need >= 1
        InvalidQuantity,
        /// Can not destroy class
        CannotDestroyClass,
        /// Attributes too large
        AttributesTooLarge,
    }
    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    //#[pallet::metadata(T::AccountId = "AccountId", ClassIdOf<T> = "ClassId", TokenIdOf<T> = "TokenId", T::Hash = "Hash")]
    pub enum Event<T: Config> {
        /// Created NFT class: \[owner, class_id\]
        CreatedClass(T::AccountId, ClassIdOf<T>),
        /// Minted NFT: \[from, to, group_id, class_id, token_id,  quantity\]
        MintedToken(
            T::AccountId,
            T::AccountId,
            GroupIdOf<T>,
            ClassIdOf<T>,
            TokenIdOf<T>,
            u32,
        ),
        /// Transferred NFT: \[from, to, class_id, token_id\]
        TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
        /// Burned NFT: \[owner, class_id, token_id\]
        BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
        /// Destroyed NFT: \[owner, class_id\]
        DestroyedClass(T::AccountId, ClassIdOf<T>),
    }
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create NFT class
        ///
        /// - `metadata`: external metadata
        #[pallet::weight(<T as Config>::WeightInfo::create_class())]
        #[transactional]
        pub fn create_class(
            origin: OriginFor<T>,
            attributes: Attributes,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            let next_id = bholdus_support_nft::Pallet::<T>::next_class_id();
            // let owner: T::AccountId = T::PalletId::get().into_sub_account(next_id);
            let data = ClassData { attributes };
            bholdus_support_nft::Pallet::<T>::create_class(&owner, data)?;
            Self::deposit_event(Event::CreatedClass(owner, next_id));
            Ok(().into())
        }

        /// Mint NFT token
        ///
        /// - `to`: the token owner's account
        /// - `class_id`: token belong to the class id
        /// - `metadata`: external metadata
        /// - `quantity`: token quantity
        #[pallet::weight(<T as Config>::WeightInfo::mint(*quantity))]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            to: <T::Lookup as StaticLookup>::Source,
            class_id: ClassIdOf<T>,
            metadata: CID,
            attributes: Attributes,
            quantity: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            Self::do_mint(who, to, class_id, metadata, attributes, quantity)
        }

        /// Transfer NFT to another account
        /// - `to` the token owner's account
        /// - `token`: (class_id, token_id)

        #[pallet::weight(<T as Config>::WeightInfo::transfer())]
        #[transactional]
        pub fn transfer(
            origin: OriginFor<T>,
            to: <T::Lookup as StaticLookup>::Source,
            token: (ClassIdOf<T>, TokenIdOf<T>),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            Self::do_transfer(&who, &to, token)
        }

        /// Burn NFT
        ///
        /// - `token`: (class_id, token_id)
        #[pallet::weight(<T as Config>::WeightInfo::burn())]
        #[transactional]
        pub fn burn(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_burn(who, token)
        }

        /// Destroy NFT class
        ///
        /// - `class_id`: The class ID to destroy
        #[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
        #[transactional]
        pub fn destroy_class(
            origin: OriginFor<T>,
            class_id: ClassIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let class_info = bholdus_support_nft::Pallet::<T>::classes(class_id)
                .ok_or(Error::<T>::ClassIdNotFound)?;
            ensure!(who == class_info.owner, Error::<T>::NoPermission);
            ensure!(
                class_info.total_issuance == Zero::zero(),
                Error::<T>::CannotDestroyClass
            );
            bholdus_support_nft::Pallet::<T>::destroy_class(&who, class_id)?;

            Self::deposit_event(Event::DestroyedClass(who, class_id));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    #[require_transactional]
    fn do_transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        token: (ClassIdOf<T>, TokenIdOf<T>),
    ) -> DispatchResult {
        let class_info = bholdus_support_nft::Pallet::<T>::classes(token.0)
            .ok_or(Error::<T>::ClassIdNotFound)?;
        let token_info = bholdus_support_nft::Pallet::<T>::tokens(token.0, token.1)
            .ok_or(Error::<T>::TokenIdNotFound)?;
        bholdus_support_nft::Pallet::<T>::transfer(from, to, token)?;

        Self::deposit_event(Event::TransferredToken(
            from.clone(),
            to.clone(),
            token.0,
            token.1,
        ));
        Ok(())
    }
    #[require_transactional]
    fn do_mint(
        who: T::AccountId,
        to: T::AccountId,
        class_id: ClassIdOf<T>,
        metadata: CID,
        attributes: Attributes,
        quantity: u32,
    ) -> DispatchResult {
        ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
        let class_info = bholdus_support_nft::Pallet::<T>::classes(class_id)
            .ok_or(Error::<T>::ClassIdNotFound)?;
        // ensure!(who == class_info.owner, Error::<T>::NoPermission);
        //
        ensure!(
            quantity <= T::MaxQuantity::get(),
            Error::<T>::InvalidQuantity
        );

        let data = TokenData { attributes };

        let group_id = bholdus_support_nft::Pallet::<T>::next_group_id();
        bholdus_support_nft::Pallet::<T>::create_group();

        let token_id = bholdus_support_nft::Pallet::<T>::next_token_id();

        for _ in 0..quantity {
            bholdus_support_nft::Pallet::<T>::mint_to_group(
                &to,
                class_id,
                group_id,
                metadata.clone(),
                data.clone(),
            )?;
        }

        Self::deposit_event(Event::MintedToken(
            who, to, group_id, class_id, token_id, quantity,
        ));
        Ok(())
    }

    fn do_burn(who: T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
        let class_info = bholdus_support_nft::Pallet::<T>::classes(token.0)
            .ok_or(Error::<T>::ClassIdNotFound)?;

        let token_info = bholdus_support_nft::Pallet::<T>::tokens(token.0, token.1)
            .ok_or(Error::<T>::TokenIdNotFound)?;
        ensure!(who == token_info.owner, Error::<T>::NoPermission);

        bholdus_support_nft::Pallet::<T>::burn(&who, token)?;

        Self::deposit_event(Event::BurnedToken(who, token.0, token.1));
        Ok(())
    }

    fn check_attributes(attributes: &Attributes) -> DispatchResult {
        // Addition can't overflow because we will be out of memory before that
        let attributes_len = attributes.iter().fold(0, |acc, (k, v)| {
            acc.saturating_add(v.len().saturating_add(k.len()) as u32)
        });
        ensure!(
            attributes_len <= T::MaxAttributesBytes::get(),
            Error::<T>::AttributesTooLarge
        );
        Ok(())
    }
}

impl<T: Config> NFT<T::AccountId> for Pallet<T> {
    type ClassId = ClassIdOf<T>;
    type TokenId = TokenIdOf<T>;
    type Balance = NFTBalance;

    fn balance(who: &T::AccountId) -> Self::Balance {
        bholdus_support_nft::TokensByOwner::<T>::iter_prefix((who,)).count() as u128
    }

    fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<T::AccountId> {
        bholdus_support_nft::Pallet::<T>::tokens(token.0, token.1).map(|t| t.owner)
    }

    fn transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        token: (Self::ClassId, Self::TokenId),
    ) -> DispatchResult {
        Self::do_transfer(from, to, token)
    }
}
