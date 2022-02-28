use super::*;
use scale_info::prelude::string::String;
use sp_std::if_std;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    pub fn do_create(
        owner: &T::AccountId,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
        beneficiary: &T::AccountId,
    ) -> DispatchResult {
        let supply = Zero::zero();
        let min_balance = Zero::zero();
        ensure!(
            Self::is_valid_symbol(symbol.clone()),
            Error::<T, I>::InvalidSymbol
        );
        ensure!(
            decimals.clone() <= T::MaxDecimals::get() as u8,
            Error::<T, I>::InvalidDecimals
        );

        let blacklist = AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone()));
        ensure!(
            !AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone())),
            Error::<T, I>::AssetBlacklist
        );

        let bounded_name: BoundedVec<u8, T::StringLimit> = Self::get_name(name.clone())
            .clone()
            .try_into()
            .map_err(|_| Error::<T, I>::BadMetadata)?;
        let bounded_symbol: BoundedVec<u8, T::StringLimit> = symbol
            .clone()
            .try_into()
            .map_err(|_| Error::<T, I>::BadMetadata)?;

        let token_id =
            NextAssetId::<T, I>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T, I>::NoAvailableTokenId)?;
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
                issuer: owner.clone(),
                admin: owner.clone(),
                freezer: owner.clone(),
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
            Metadata::<T, I>::insert(token_id, metadata.clone());
            Ok(())
        })?;
        Ok(())
    }

    pub fn do_create_and_mint(
        owner: &T::AccountId,
        admin: &T::AccountId,
        name: Vec<u8>,
        symbol: Vec<u8>,
        decimals: u8,
        beneficiary: &T::AccountId,
        supply: T::Balance,
        min_balance: T::Balance,
    ) -> DispatchResult {
        if supply.is_zero() {
            return Ok(());
        }
        ensure!(
            Self::is_valid_symbol(symbol.clone()),
            Error::<T, I>::InvalidSymbol
        );
        ensure!(!min_balance.is_zero(), Error::<T, I>::MinBalanceZero);
        ensure!(
            decimals.clone() <= T::MaxDecimals::get() as u8,
            Error::<T, I>::InvalidDecimals
        );

        let blacklist = AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone()));
        ensure!(
            !AssetsBlacklist::<T, I>::get().contains(&(name.clone(), symbol.clone())),
            Error::<T, I>::AssetBlacklist
        );

        let bounded_name: BoundedVec<u8, T::StringLimit> = Self::get_name(name.clone())
            .clone()
            .try_into()
            .map_err(|_| Error::<T, I>::BadMetadata)?;
        let bounded_symbol: BoundedVec<u8, T::StringLimit> = symbol
            .clone()
            .try_into()
            .map_err(|_| Error::<T, I>::BadMetadata)?;

        let token_id =
            NextAssetId::<T, I>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T, I>::NoAvailableTokenId)?;
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
                owner: admin.clone(),
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
            Metadata::<T, I>::insert(token_id, metadata.clone());
            Ok(())
        })?;
        Ok(())
    }
}
