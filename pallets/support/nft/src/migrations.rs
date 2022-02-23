use frame_support::{
    dispatch::{GetStorageVersion, Weight},
    log,
    storage::migration,
};

use crate::*;

pub fn migrate<T: crate::Config>() -> Weight {
    migration::move_pallet(b"SupportNFT", b"BholdusSupportNFT");
    0
}
