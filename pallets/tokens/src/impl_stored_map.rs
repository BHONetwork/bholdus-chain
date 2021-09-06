//! Tokens pallet's `StoredMap` implementation.

use super::*;
impl<T: Config<I>, I: 'static> StoredMap<(T::AssetId, T::AccountId), T::Extra> for Pallet<T, I> {
    fn get(id_who: &(T::AssetId, T::AccountId)) -> T::Extra {
        let &(id, ref who) = id_who;
        if Account::<T, I>::contains_key(id, who) {
            Account::<T, I>::get(id, who).extra
        } else {
            Default::default()
        }
    }

    fn try_mutate_exists<R, E: From<StoredMapError>>(
        id_who: &(T::AssetId, T::AccountId),
        f: impl FnOnce(&mut Option<T::Extra>) -> Result<R, E>,
    ) -> Result<R, E> {
        let &(id, ref who) = id_who;
        let mut maybe_extra = Some(Account::<T, I>::get(id, who).extra);
        let r = f(&mut maybe_extra)?;
        // They want to write some value or delete it.
        // If the account existed and they want to write a value, then we write.
        // If the account didn't exist and they want to delete it, then we let it pass.
        // Otherwis, we fail.
        Account::<T, I>::try_mutate_exists(id, who, |maybe_account| {
            if let Some(extra) = maybe_extra {
                // They want to write a value. Let this happen only if the account actually exists.
                if let Some(ref mut account) = maybe_account {
                    account.extra = extra;
                } else {
                    Err(StoredMapError::NoProviders)?;
                }
            } else {
                // They want to delete it. Let this pass if the item never existed anyway.
                ensure!(maybe_account.is_none(), StoredMapError::ConsumerRemaining);
            }
            Ok(r)
        })
    }
}