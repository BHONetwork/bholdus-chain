#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_currencies.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> bholdus_currencies::WeightInfo for WeightInfo<T> {
    fn transfer_non_native_currency() -> Weight {
        (123_019_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    fn transfer_native_currency() -> Weight {
        (19_885_000 as Weight)
    }
    fn update_balance_non_native_currency() -> Weight {
        (57_645_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn update_balance_native_currency_creating() -> Weight {
        (24_430_000 as Weight)
    }
    fn update_balance_native_currency_killing() -> Weight {
        (24_402_000 as Weight)
    }
}
