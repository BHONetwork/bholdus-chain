use frame_support::assert_ok;

use std::{assert_matches::assert_matches, str::from_utf8};

use crate::mock::*;
use crate::*;
use fp_evm::{Context, PrecompileFailure};
use sha3::{Digest, Keccak256};

fn precompiles() -> PalletTemplatePrecompileSet<Runtime> {
    PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
    ExtBuilder::default().build().execute_with(|| {
        // This selector is only three bytes long when four are required.
        let bogus_selector = vec![1u8, 2u8, 3u8];

        assert_matches!(
            precompiles().execute(
                Account::Alice.into(),
                &bogus_selector,
                None,
                &Context {
                    address: Account::Alice.into(),
                    caller: Account::Alice.into(),
                    apparent_value: From::from(0u32),
                },
                false,
            ),
            Some(Err(PrecompileFailure::Revert { output, .. }))
            if output == b"tried to parse selector out of bounds"
        );
    });
}

#[test]
fn no_selector_exists_but_length_is_right() {
    ExtBuilder::default().build().execute_with(|| {
        let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

        assert_matches!(
            precompiles().execute(
                Account::Alice.into(),
                &bogus_selector,
                None,
                &Context {
                    address: Account::Alice.into(),
                    caller: Account::Alice.into(),
                    apparent_value: From::from(0),
                },
                false,
            ),
            Some(Err(PrecompileFailure::Revert { output, ..}))
                if output == b"unknown selector",
        );
    });
}

#[test]
fn selectors() {
    // These selectors are only template values and not correct.
    // Change these selector values to the correct to pass the test.
    // assert_eq!(Action::DoSomething as u32, 0x70a08231);
    // assert_eq!(Action::GetValue as u32, 0x18160ddd);

    assert_eq!(
        crate::SELECTOR_LOG_SOMETHING,
        &Keccak256::digest(b"Something(uint256)")[..]
    );
}
