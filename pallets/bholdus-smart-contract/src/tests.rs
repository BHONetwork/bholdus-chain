use crate::{mock::*, ContractEntry};
use frame_support::{assert_noop, assert_ok};

#[test]
fn stores_value() {
	let origin = Origin::signed(ALICE);
	let chain_extension_input = 5;
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TemplateModule::insert_number(origin, chain_extension_input));
		assert_eq!(ContractEntry::<Test>::get(), chain_extension_input);
	})
}

#[test]
fn rejects_large_input() {
	let origin = Origin::signed(ALICE);
	let large_selector = [0; crate::MAX_LENGTH].to_vec();
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			TemplateModule::call_smart_contract(origin, ALICE, large_selector, 5, 100000000),
			crate::Error::<Test>::InputTooLarge
		);
	})
}
