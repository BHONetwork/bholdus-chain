use frame_support::dispatch::{GetStorageVersion, Weight};
use frame_support::storage::migration;

use crate::*;

pub fn migrate<T: crate::Config>() -> Weight {
    let storage_version = Pallet::<T>::on_chain_storage_version();
    let mut weight: Weight = 0;
    if storage_version < 2 {
        weight = weight.saturating_add(v2::migrate_to_v2::<T>());
        StorageVersion::new(2).put::<Pallet<T>>();
    }
    weight
}

pub mod v2 {
    use super::*;

    pub fn migrate_to_v2<T: crate::Config>() -> Weight {
        ServiceFee::<T>::put(0);
        migration::remove_storage_prefix(Pallet::<T>::name().as_bytes(), b"ServiceFeeRate", b"");
        T::DbWeight::get().writes(2)
    }
}
