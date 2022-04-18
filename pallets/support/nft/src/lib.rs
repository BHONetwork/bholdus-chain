//! # Non Fungible Token

//! ### Functions
//!
//! - `create_clas` - Create NFT(non fungible token) class
//! - `transfer` - Transfer NFT to another account.
//! - `mint` - Mint NFT
//! - `burn` - Burn NFT
//! - `destroy_class` - Destroy NFT

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    ensure, pallet_prelude::*, require_transactional, traits::Get, transactional, BoundedVec,
    Parameter,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero,
    },
    ArithmeticError, DispatchError, DispatchResult, RuntimeDebug,
};

use sp_std::{convert::TryInto, if_std, vec::Vec};

use frame_support::traits::StorageVersion;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct ClassInfo<TokenId, AccountId, Data> {
    // /// Class metadata
    // pub metadata: ClassMetadataOf,
    /// Total issuance for the class
    pub total_issuance: TokenId,
    /// Class owner
    pub owner: AccountId,
    /// Class Properties
    pub data: Data,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<AccountId, Data, TokenMetadataOf> {
    /// Token metadata
    pub metadata: TokenMetadataOf,
    /// Token owner
    pub owner: AccountId,
    /// Token creator,
    pub creator: AccountId,
    /// Token Properties
    pub data: Data,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The class ID type
        type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The group ID type
        type GroupId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The token ID type
        type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The class properties type
        type ClassData: Parameter + Member + MaybeSerializeDeserialize;
        /// The token properties type
        type TokenData: Parameter + Member + MaybeSerializeDeserialize;
        /// The maximum size of a class's metadata
        type MaxClassMetadata: Get<u32>;

        /// The maximum size of a token's metadata
        type MaxTokenMetadata: Get<u32>;
    }

    pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;
    pub type TokenMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxTokenMetadata>;
    pub type ClassInfoOf<T> = ClassInfo<
        <T as Config>::TokenId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::ClassData,
    >;
    pub type TokenInfoOf<T> = TokenInfo<
        <T as frame_system::Config>::AccountId,
        <T as Config>::TokenData,
        TokenMetadataOf<T>,
    >;

    pub type GenesisTokenData<T> = (
        <T as frame_system::Config>::AccountId, // Token owner
        Vec<u8>,                                // Token metadata
        <T as Config>::TokenData,
    );
    pub type GenesisTokens<T> = (
        <T as frame_system::Config>::AccountId, // Token class owner
        <T as Config>::ClassData,
        Vec<GenesisTokenData<T>>, // Vector of tokens belonging to this class
    );

    /// Error for non-fungible-token module.
    #[pallet::error]
    pub enum Error<T> {
        /// No available class ID
        NoAvailableClassId,
        /// No available group ID
        NoAvailableGroupId,
        /// No available token ID
        NoAvailableTokenId,
        /// Lock NFT
        IsLocked,
        /// Token(ClassId, TokenId) not found
        TokenNotFound,
        /// Class not found
        ClassNotFound,
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Can not destroy class
        /// Total issuance is not 0
        CannotDestroyClass,
        /// Can not unlock token
        AlreadyUnlocked,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
    }

    /// Next available class ID.
    #[pallet::storage]
    #[pallet::getter(fn next_class_id)]
    pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

    /// Next available group ID.
    #[pallet::storage]
    #[pallet::getter(fn next_group_id)]
    pub type NextGroupId<T: Config> = StorageValue<_, T::GroupId, ValueQuery>;

    /// Next available token ID.
    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    pub type NextTokenId<T: Config> = StorageValue<_, T::TokenId, ValueQuery>;

    /// Next available token ID.
    #[pallet::storage]
    #[pallet::getter(fn next_token_class_id)]
    pub type NextTokenIdByClass<T: Config> =
        StorageMap<_, Twox64Concat, T::ClassId, T::TokenId, ValueQuery>;

    /// Store class info.
    ///
    /// Returns `None` if class info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn classes)]
    pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

    /// Store token info.
    ///
    /// Returns `None` if token info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub type Tokens<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TokenInfoOf<T>>;

    /// Token existence check by owner and class ID.
    #[pallet::storage]
    #[pallet::getter(fn tokens_by_owner)]
    pub type TokensByOwner<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        (T::AccountId, T::TokenId),
        ValueQuery,
    >;

    /// Store group info
    #[pallet::storage]
    #[pallet::getter(fn tokens_by_group)]
    pub type TokensByGroup<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::GroupId>, // group id
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        T::TokenId,
        //ValueQuery,
    >;

    /// Lock NFT on account
    #[pallet::storage]
    #[pallet::getter(fn set_lock)]
    pub type LockableNFT<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        (),
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: Vec<GenesisTokens<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig { tokens: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.tokens.iter().for_each(|token_class| {
                let class_id = Pallet::<T>::create_class(&token_class.0, token_class.1.clone())
                    .expect("Create class cannot fail while building genesis");
                for (account_id, token_metadata, token_data) in &token_class.2 {
                    Pallet::<T>::mint(
                        account_id,
                        class_id,
                        token_metadata.to_vec(),
                        token_data.clone(),
                    )
                    .expect("Token mint cannot fail during genesis");
                }
            })
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create NFT(non fungible token) class
    pub fn create_class(
        owner: &T::AccountId,
        // metadata: Vec<u8>,
        data: T::ClassData,
    ) -> Result<T::ClassId, DispatchError> {
        // let bounded_metadata: BoundedVec<u8, T::MaxClassMetadata> = metadata
        //     .try_into()
        //     .map_err(|_| Error::<T>::MaxMetadataExceeded)?;

        let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableClassId)?;
            Ok(current_id)
        })?;

        let info = ClassInfo {
            // metadata: bounded_metadata,
            total_issuance: Default::default(),
            owner: owner.clone(),
            data,
        };
        Classes::<T>::insert(class_id, info);

        Ok(class_id)
    }

    /// Create group
    pub fn create_group() -> Result<T::GroupId, DispatchError> {
        let group_id = NextGroupId::<T>::try_mutate(|id| -> Result<T::GroupId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableGroupId)?;
            Ok(current_id)
        })?;
        Ok(group_id)
    }

    /// Transfer NFT
    pub fn transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        token: (T::ClassId, T::TokenId),
    ) -> DispatchResult {
        Tokens::<T>::try_mutate(token.0, token.1, |token_info| -> DispatchResult {
            let mut info = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(info.owner == *from, Error::<T>::NoPermission);
            ensure!(
                !Self::is_lock(&info.owner, (token.0, token.1)),
                Error::<T>::IsLocked
            );
            if from == to {
                // no change needed
                return Ok(());
            }
            info.owner = to.clone();

            TokensByOwner::<T>::remove((from, token.0, token.1));

            TokensByOwner::<T>::insert((to, token.0, token.1), (to, token.1));

            Ok(())
        })
    }

    /// Mint NFT to `owner`
    pub fn mint(
        owner: &T::AccountId,
        class_id: T::ClassId,
        metadata: Vec<u8>,
        data: T::TokenData,
    ) -> Result<T::TokenId, DispatchError> {
        NextTokenIdByClass::<T>::mutate(class_id, |id| -> Result<T::TokenId, DispatchError> {
            let bounded_metadata: BoundedVec<u8, T::MaxTokenMetadata> = metadata
                .try_into()
                .map_err(|_| Error::<T>::MaxMetadataExceeded)?;
            let token_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableTokenId)?;
            Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
                let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_add(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            let token_info = TokenInfo {
                metadata: bounded_metadata,
                owner: owner.clone(),
                creator: owner.clone(),
                data,
            };

            Tokens::<T>::insert(class_id, token_id, token_info);
            TokensByOwner::<T>::insert((owner, class_id, token_id), (owner, token_id));
            Ok(token_id)
        })
    }

    /// Mint NFT to `owner`
    pub fn mint_to_group(
        owner: &T::AccountId,
        class_id: T::ClassId,
        group_id: T::GroupId,
        metadata: Vec<u8>,
        data: T::TokenData,
    ) -> Result<T::TokenId, DispatchError> {
        NextTokenIdByClass::<T>::try_mutate(
            class_id,
            |class_token_id| -> Result<T::TokenId, DispatchError> {
                NextTokenId::<T>::try_mutate(|id| -> Result<T::TokenId, DispatchError> {
                    let bounded_metadata: BoundedVec<u8, T::MaxTokenMetadata> = metadata
                        .try_into()
                        .map_err(|_| Error::<T>::MaxMetadataExceeded)?;
                    let token_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::NoAvailableTokenId)?;

                    *class_token_id = token_id;

                    Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
                        let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
                        info.total_issuance = info
                            .total_issuance
                            .checked_add(&One::one())
                            .ok_or(ArithmeticError::Overflow)?;
                        Ok(())
                    })?;

                    let token_info = TokenInfo {
                        metadata: bounded_metadata,
                        owner: owner.clone(),
                        creator: owner.clone(),
                        data,
                    };
                    /*if_std!(println!(
                        "SupportNFT class_id-token_id-class_token_id {:?}:{:?}:{:?}",
                        class_id, token_id, class_token_id
                    ));
                    */

                    Tokens::<T>::insert(class_id, token_id, token_info);
                    TokensByOwner::<T>::insert((owner, class_id, token_id), (owner, token_id));
                    TokensByGroup::<T>::insert((group_id, class_id, token_id), token_id);
                    Ok(token_id)
                });
                Ok(*class_token_id)
            },
        )
    }

    /// Burn NFT
    pub fn burn(owner: &T::AccountId, token: (T::ClassId, T::TokenId)) -> DispatchResult {
        Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
            let t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(t.owner == *owner, Error::<T>::NoPermission);

            ensure!(
                !Self::is_lock(&t.owner, (token.0, token.1)),
                Error::<T>::IsLocked
            );

            Classes::<T>::try_mutate(token.0, |class_info| -> DispatchResult {
                let info = class_info.as_mut().ok_or(Error::<T>::ClassNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_sub(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;

            TokensByOwner::<T>::remove((owner, token.0, token.1));
            Ok(())
        })
    }

    /// Destroy NFT
    pub fn destroy_class(owner: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
        Classes::<T>::try_mutate_exists(class_id, |class_info| -> DispatchResult {
            let info = class_info.take().ok_or(Error::<T>::ClassNotFound)?;
            ensure!(info.owner == *owner, Error::<T>::NoPermission);
            ensure!(
                info.total_issuance == Zero::zero(),
                Error::<T>::CannotDestroyClass
            );

            NextTokenIdByClass::<T>::remove(class_id);
            Ok(())
        })
    }
}

impl<T: Config> Pallet<T> {
    pub fn owner(token: (T::ClassId, T::TokenId)) -> T::AccountId {
        Tokens::<T>::get(token.0, token.1).unwrap().owner
    }

    pub fn lock(account: &T::AccountId, token: (T::ClassId, T::TokenId)) {
        LockableNFT::<T>::insert((account, token.0, token.1), ())
    }

    pub fn unlock(owner: &T::AccountId, token: (T::ClassId, T::TokenId)) {
        LockableNFT::<T>::remove((owner, token.0, token.1))
    }

    pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
        TokensByOwner::<T>::contains_key((account, token.0, token.1))
    }

    pub fn is_lock(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
        LockableNFT::<T>::contains_key((account, token.0, token.1))
    }
}
