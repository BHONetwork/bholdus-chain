use super::{BoundedVec, ChainId, MemoInfo, TryInto, TxnHash};
use crate::mock::*;
use frame_support::assert_ok;
#[test]
fn create_should_work() {
    ExtBuilder::default()
        .with_balances(vec![(ALICE, 10), (BOB, 10)])
        .build()
        .execute_with(|| {
            let content: Vec<u8> = String::from("TEST").into_bytes();
            let chain_id: ChainId = 10;
            let txn_hash: TxnHash = String::from("HASH").into_bytes();
            let operator = ALICE;
            let sender = String::from("ALICE").into_bytes();
            let receiver = String::from("BOB").into_bytes();

            assert_ok!(Memo::create(
                Origin::signed(ALICE),
                chain_id,
                txn_hash,
                content,
                sender,
                receiver,
            ));

            let content: Vec<u8> = String::from("TEST").into_bytes();
            let txn_hash: TxnHash = String::from("HASH").into_bytes();
            let sender = String::from("ALICE").into_bytes();
            let receiver = String::from("BOB").into_bytes();

            let memo_created = Memo::memo(chain_id, &txn_hash).unwrap();

            let bounded_content: BoundedVec<u8, ContentLimit> = content.clone().try_into().unwrap();

            let memo_input = MemoInfo {
                content: bounded_content,
                sender,
                receiver,
                operator,
                time: memo_created.time,
            };
            assert_eq!(memo_input, memo_created);

            System::assert_last_event(Event::Memo(crate::Event::MemoCreated(
                chain_id, txn_hash, memo_input,
            )));
        })
}

#[test]
fn set_amount_free_tx() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Memo::set_amount_free_tx(Origin::root(), 100));

        assert_eq!(Memo::amount_free_tx(), 100);
    })
}

// #[test]
// fn update_should_work() {
//     ExtBuilder::default()
//         .with_balances(vec![(ALICE, 10), (BOB, 10)])
//         .build()
//         .execute_with(|| {
//             let content: Vec<u8> = String::from("TEST").into_bytes();
//             let chain_id: ChainId = 10;
//             let txn_hash: TxnHash = String::from("HASH").into_bytes();
//             let operator = ALICE;
//             let sender = ALICE;
//             let receiver = BOB;

//             assert_ok!(Memo::create(
//                 Origin::signed(ALICE),
//                 chain_id,
//                 txn_hash,
//                 content,
//                 sender,
//                 receiver,
//             ));

//             let new_content: Vec<u8> = String::from("TEST_UPDATE").into_bytes();
//             let txn_hash: TxnHash = String::from("HASH").into_bytes();

//             assert_ok!(Memo::update(
//                 Origin::signed(ALICE),
//                 chain_id,
//                 txn_hash,
//                 new_content,
//             ));

//             let new_content: Vec<u8> = String::from("TEST_UPDATE").into_bytes();
//             let txn_hash: TxnHash = String::from("HASH").into_bytes();

//             let memo_updated = Memo::memo(chain_id, &txn_hash).unwrap();

//             let memo_input = MemoInfo {
//                 content: new_content,
//                 sender,
//                 receiver,
//                 operator,
//                 time: memo_updated.time,
//             };
//             assert_eq!(memo_input, memo_updated);

//             System::assert_last_event(Event::Memo(crate::Event::MemoUpdated(
//                 chain_id, txn_hash, memo_input,
//             )));
//         })
// }
