use frame_support::{
    dispatch::{GetStorageVersion, Weight},
    log,
    storage::migration,
};

use crate::*;

pub fn migrate<T: crate::Config>() -> Weight {
    log::info!("Support NFT migrates pallet name from SupportNFT to BholdusSupportNF");
    migration::move_pallet(b"SupportNFT", b"BholdusSupportNFT");
    0
}
