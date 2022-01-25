#![cfg_attr(not(feature = "std"), no_std)]
pub use self::lixi::LixiApp;
use ink_env::{AccountId, Environment, Error, Hash};
use ink_lang as ink;

#[ink::chain_extension]
pub trait LixiChainExtension {
    type ErrorCode = ContractError;
    #[ink(extension = 1, returns_result = false)]
    fn claim(
        from: AccountId,
        recipient: AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), ContractError>;

    #[ink(extension = 2, returns_result = false)]
    fn fetch_random(subject: [u8; 32]) -> [u8; 32];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    UnknownStatusCode,
    InvalidScaleEncoding,
    InvalidRequest,
    UnavailableReward,
    MaxReward,
    Overflow,
}

impl From<scale::Error> for ContractError {
    fn from(_: scale::Error) -> Self {
        ContractError::InvalidScaleEncoding
    }
}

impl ink_env::chain_extension::FromStatusCode for ContractError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            _ => Err(Self::UnknownStatusCode),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum LixiEnv {}

impl Environment for LixiEnv {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;
    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    type ChainExtension = LixiChainExtension;
}

#[ink::contract(env = crate::LixiEnv)]
/// A smart contract with a custom environment, necessary for the chain extension
pub mod lixi {
    use super::*;
    use core::convert::TryInto;
    use ink_prelude::string::String;
    use ink_prelude::vec;
    use ink_prelude::vec::Vec;
    use ink_storage::lazy::Mapping;
    use ink_storage::{
        collections::{hashmap::Entry, HashMap as StorageHashMap, Vec as StorageVec},
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };
    type DayId = i8;
    type RewardType = i8;
    type Quantity = u32;
    pub const HOLIDAY_1: DayId = 1;
    pub const HOLIDAY_2: DayId = 2;
    pub const HOLIDAY_3: DayId = 3;
    pub const REWARD_TYPE_1: RewardType = 1;
    pub const REWARD_TYPE_2: RewardType = 2;
    pub const REWARD_TYPE_3: RewardType = 3;
    pub const REWARD_TYPE_4: RewardType = 4;
    pub const HOLIDAY_1_BLOCK_NUMBER: BlockNumber = 1083296;
    pub const HOLIDAY_2_BLOCK_NUMBER: BlockNumber = 1112096;
    pub const HOLIDAY_3_BLOCK_NUMBER: BlockNumber = 1140896;
    pub const END_OF_HOLIDAY: BlockNumber = 1169696;
    pub const REWARD_1_VALUE: Balance = 99999;
    pub const REWARD_2_VALUE: Balance = 2000;
    pub const REWARD_3_VALUE: Balance = 500;
    pub const REWARD_4_VALUE: Balance = 0;
    pub const MAX_REWARD_1: Quantity = 1;
    pub const MAX_REWARD_2: Quantity = 204;
    pub const MAX_REWARD_3: Quantity = 333;
    pub const MAX_REWARD_4: Quantity = u32::MAX;
    const WRONG_DAY: &str = "Invalid transaction. Abort";

    #[ink(storage)]
    pub struct LixiApp {
        // use for testing, set blocknumber
        pub block: BlockNumber,
        // use for randomness algorithm
        pub nonce: u32,
        // (AccountId, DayId)
        pub winners: StorageHashMap<(AccountId, DayId), (Timestamp, Balance)>,
        // (DayId, RewardType)
        pub reward_per_day: StorageHashMap<(DayId, RewardType), Quantity>,
    }

    /// Event emitted when user claimed BHO
    #[ink(event)]
    pub struct Reward {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
        timestamp: Timestamp,
    }

    impl LixiApp {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                block: Default::default(),
                nonce: Default::default(),
                winners: StorageHashMap::default(),
                reward_per_day: StorageHashMap::default(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        #[inline]
        pub fn work_with_reward_1(&self, day_id: DayId) -> (RewardType, Balance, &Quantity) {
            let curr_quantity = self
                .reward_per_day
                .get(&(day_id, REWARD_TYPE_1))
                .unwrap_or(&0u32);
            if curr_quantity == &MAX_REWARD_1 {
                // TODO:
                self.work_with_reward_2(day_id)
            } else {
                // available REWARD_TYPE_2
                (REWARD_TYPE_1, REWARD_1_VALUE, curr_quantity)
            }
        }

        #[inline]
        pub fn work_with_reward_2(&self, day_id: DayId) -> (RewardType, Balance, &Quantity) {
            let type_of_reward = REWARD_TYPE_2;
            let max_quantity = &MAX_REWARD_2;
            let value_of_reward = REWARD_2_VALUE;

            let curr_quantity = self
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .unwrap_or(&0u32);
            if curr_quantity < max_quantity {
                // available REWARD_TYPE_2
                (type_of_reward, value_of_reward, curr_quantity)
            } else {
                // TODO:
                // unavailable reward 2
                // check REWARD_TYPE_3
                let quantity_of_reward_3 = self
                    .reward_per_day
                    .get(&(day_id, REWARD_TYPE_3))
                    .unwrap_or(&0u32);
                if quantity_of_reward_3 == &MAX_REWARD_3 {
                    // TODO: return reward 4
                    let quantity_of_reward_4 = self
                        .reward_per_day
                        .get(&(day_id, REWARD_TYPE_4))
                        .unwrap_or(&0u32);
                    (REWARD_TYPE_4, REWARD_4_VALUE, quantity_of_reward_4)
                } else {
                    // available REWARD_TYPE_3
                    (REWARD_TYPE_3, REWARD_3_VALUE, quantity_of_reward_3)
                }
            }
        }

        #[inline]
        pub fn work_with_reward_3(&self, day_id: DayId) -> (RewardType, Balance, &Quantity) {
            let type_of_reward = REWARD_TYPE_3;
            let max_quantity = &MAX_REWARD_3;
            let value_of_reward = REWARD_3_VALUE;
            let curr_quantity = self
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .unwrap_or(&0u32);

            if curr_quantity < max_quantity {
                // Available reward type 3
                (type_of_reward, value_of_reward, curr_quantity)
            } else {
                let quantity_of_reward_2 = self
                    .reward_per_day
                    .get(&(day_id, REWARD_TYPE_2))
                    .unwrap_or(&0u32);
                if quantity_of_reward_2 == &MAX_REWARD_2 {
                    // TODO: return reward
                    let quantity_of_reward_4 = self
                        .reward_per_day
                        .get(&(day_id, REWARD_TYPE_4))
                        .unwrap_or(&0u32);
                    (REWARD_TYPE_4, REWARD_4_VALUE, quantity_of_reward_4)
                } else {
                    // available REWARD_TYPE_2
                    (REWARD_TYPE_2, REWARD_2_VALUE, quantity_of_reward_2)
                }
            }
        }

        #[inline]
        pub fn recursion(
            &self,
            reward_type: (RewardType, Balance, Quantity),
            day_id: DayId,
        ) -> (RewardType, Balance, &Quantity) {
            match reward_type {
                (1, value, max_quantity) => {
                    if day_id == HOLIDAY_1 {
                        // unavailable reward 1 in 1st day
                        self.work_with_reward_2(day_id)
                    } else if day_id == HOLIDAY_2 {
                        self.work_with_reward_1(day_id)
                    } else {
                        // holiday = 3
                        // ensure no user claim reward 1 in 2nd day
                        assert!(!self
                            .reward_per_day
                            .contains_key(&(HOLIDAY_2, REWARD_TYPE_1)));
                        self.work_with_reward_1(day_id)
                    }
                }
                (2, value, max_quantity) => self.work_with_reward_2(day_id),
                (3, value, max_quantity) => self.work_with_reward_3(day_id),
                // (4, value, max_quantity)
                (type_of_reward, value, max_quantity) => {
                    let quantity = self
                        .reward_per_day
                        .get(&(day_id, type_of_reward))
                        .unwrap_or(&0u32);
                    (type_of_reward, value, quantity)
                }
            }
        }

        /// Lixi App
        #[ink(message)]
        pub fn lixi(&mut self) -> Result<Balance, ContractError> {
            let caller: AccountId = self.env().caller();
            let account_id: AccountId = self.env().account_id();
            // let block_number: BlockNumber = self.env().block_number();
            let timestamp: Timestamp = self.env().block_timestamp();

            let subject_runtime: [u32; 32] = [self.nonce; 32];
            let subject: u32 = self.nonce;

            // #[ink::test]
            // let (_hash, random_block) = self.env().random(&[subject]);
            let block_number: BlockNumber = self.block;

            // let random_seed = self.env().extension().fetch_random(subject_runtime)?;
            // let random: &[u8] = &random_seed;
            // let random_vec: Vec<u8> = random.to_vec();
            // let index0 = random_vec[0];
            // let div = index0 % 100;
            let div = subject % 100;
            ink_env::debug_println!(
                "ink_lixi div: {:?}; subject {:?}; block_number {:?}",
                div,
                subject,
                block_number
            );
            let matched_reward = if div >= 1 && div <= 5 {
                (REWARD_TYPE_1, REWARD_1_VALUE, MAX_REWARD_1)
            } else if div >= 6 && div <= 15 {
                (REWARD_TYPE_2, REWARD_2_VALUE, MAX_REWARD_2)
            } else if div >= 16 && div <= 30 {
                (REWARD_TYPE_3, REWARD_3_VALUE, MAX_REWARD_3)
            } else if div >= 31 && div <= 100 {
                (REWARD_TYPE_4, REWARD_4_VALUE, 0)
            } else {
                return Err(ContractError::Overflow);
            };

            let (amount, day_id, (type_of_reward, num_of_rewards)) = if block_number
                >= HOLIDAY_1_BLOCK_NUMBER
                && block_number < HOLIDAY_2_BLOCK_NUMBER
            {
                // Action: M1
                let holiday: DayId = HOLIDAY_1;
                let user = self.winners.get(&(caller, holiday));
                ink_env::debug_println!("ink_lixi day: {:?}", &holiday);
                ink_env::debug_println!("ink_lixi winners: {:?}", &user);
                if user.is_some() {
                    ink_env::debug_println!("ink_lixi existed");
                    return Err(ContractError::InvalidRequest);
                };
                let matched_reward = self.recursion(matched_reward, holiday);
                match matched_reward {
                    (reward_type, value, curr_quantity) => {
                        (value, holiday, (reward_type, curr_quantity + 1))
                    }
                    _ => {
                        return Err(ContractError::Overflow);
                    }
                }
            } else if block_number >= HOLIDAY_2_BLOCK_NUMBER
                && block_number < HOLIDAY_3_BLOCK_NUMBER
            {
                // Action: M2
                let holiday: DayId = HOLIDAY_2;
                let user = self.winners.get(&(caller, holiday));
                if user.is_some() {
                    return Err(ContractError::InvalidRequest);
                };
                let matched_reward = self.recursion(matched_reward, holiday);
                match matched_reward {
                    (reward_type, value, curr_quantity) => {
                        (value, holiday, (reward_type, curr_quantity + 1))
                    }
                    // Impossible cases
                    _ => return Err(ContractError::Overflow),
                }
            } else if block_number >= HOLIDAY_3_BLOCK_NUMBER && block_number < END_OF_HOLIDAY {
                let holiday: DayId = HOLIDAY_3;
                ink_env::debug_println!("ink_lixi day: {:?}", &holiday);
                let user = self.winners.get(&(caller, holiday));
                ink_env::debug_println!("ink_lixi winners: {:?}", &user);
                if user.is_some() {
                    ink_env::debug_println!("ink_lixi existed");
                    return Err(ContractError::InvalidRequest);
                };
                self.ensure_no_winner(caller, holiday);
                match matched_reward {
                    (4, value, _) => {
                        // Don't need check quantity
                        let reward_type: i8 = 4i8;
                        let quantity = self
                            .reward_per_day
                            .get(&(holiday, reward_type))
                            .unwrap_or(&0u32);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    (1, value, max_quantity) => {
                        let reward_type: i8 = 1i8;
                        assert!(!self.reward_per_day.contains_key(&(HOLIDAY_2, reward_type)));
                        let quantity = self.reward_per_day.get(&(holiday, reward_type)).unwrap();
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    (reward_type, value, max_quantity) => {
                        let quantity = self.reward_per_day.get(&(holiday, reward_type)).unwrap();
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    _ => return Err(ContractError::Overflow),
                }
            } else {
                return Err(ContractError::Overflow);
            };

            ink_env::debug_println!(
                "ink_lixi caller{:?}; amount {:?}; day_id {:?}",
                &caller,
                &amount,
                &day_id
            );
            let reward: Balance = amount * 10u128.checked_pow(18).unwrap();
            // TODO: Do transfer
            self.env().transfer(caller, reward);
            self.nonce += 1;
            self.winners.insert((caller, day_id), (timestamp, amount));
            self.reward_per_day
                .insert((day_id, type_of_reward), num_of_rewards);

            self.env().emit_event(Reward {
                from: account_id,
                to: caller,
                value: reward,
                timestamp,
            });
            Ok(amount)
        }

        /// Only support for testing
        #[ink(message)]
        pub fn set_block_to_test(&mut self, block: BlockNumber) -> Result<(), ContractError> {
            self.block = block;
            Ok(())
        }

        #[ink(message)]
        pub fn get_block_to_test(&self) -> BlockNumber {
            self.block
        }

        /// Transfer from funds of smart contract
        #[ink(message)]
        pub fn transfer(&mut self, dest: AccountId, value: Balance) -> Result<(), ContractError> {
            self.env().transfer(dest, value);
            Ok(())
        }

        /// Call runtime to transfer
        #[ink(message)]
        pub fn runtime_transfer(&mut self, value: Balance) -> Result<(), ContractError> {
            self.env()
                .extension()
                .claim(self.env().account_id(), self.env().caller(), value);
            Ok(())
        }

        #[ink(message)]
        pub fn get_winners(&self, owner: AccountId) -> Vec<(Timestamp, Balance)> {
            let (t1, a1) = self.winners.get(&(owner, 1i8)).copied().unwrap_or((0, 0));
            let (t2, a2) = self.winners.get(&(owner, 2i8)).copied().unwrap_or((0, 0));
            let (t3, a3) = self.winners.get(&(owner, 3i8)).copied().unwrap_or((0, 0));

            let mut xs = vec![(t1, a1), (t2, a2), (t3, a3)];
            xs.retain(|&(x, _)| x != 0);
            xs
        }

        #[ink(message)]
        pub fn get_reward_per_day(&self) -> Vec<u32> {
            let day_id: DayId = 1i8;
            let a1 = self
                .reward_per_day
                .get(&(day_id, 1i8))
                .copied()
                .unwrap_or(0u32);
            let a2 = self
                .reward_per_day
                .get(&(day_id, 2i8))
                .copied()
                .unwrap_or(0u32);
            let a3 = self
                .reward_per_day
                .get(&(day_id, 3i8))
                .copied()
                .unwrap_or(0u32);
            let a4 = self
                .reward_per_day
                .get(&(day_id, 4i8))
                .copied()
                .unwrap_or(0u32);
            vec![a1, a2, a3, a4]
        }

        /// Panic if the AccountId-DayId does not exit.
        fn ensure_no_winner(&self, account_id: AccountId, day_id: DayId) {
            assert!(!self.winners.contains_key(&(account_id, day_id)));
        }

        /// Ensure available reward per day
        fn ensure_available_reward(&self, quantity: &u32, max_quantity: &u32) {
            assert!(quantity < max_quantity);
        }

        #[ink(message)]
        pub fn get_nonce(&self) -> u32 {
            self.nonce
        }

        /*#[ink(message)]
        pub fn get_vec(&self) -> Vec<(u8, u8, u8)> {
            self.vec.clone()
        }
        */

        /// Returns balance of smart contract.
        #[ink(message)]
        pub fn balance_of(&self) -> Balance {
            self.env().balance()
        }

        #[ink(message)]
        pub fn block_number(&self) -> BlockNumber {
            self.env().block_number()
        }

        #[ink(message)]
        pub fn timestamp(&self) -> Timestamp {
            self.env().block_timestamp()
        }

        /// AccountId of smart contract
        #[ink(message)]
        pub fn account_id(&self) -> AccountId {
            self.env().account_id()
        }

        // support for testing
        #[inline]
        fn return_day_id(&self) -> DayId {
            let nonce = self.nonce;
            if self.nonce >= 1 && self.nonce <= 5 {
                1i8
            } else if nonce >= 6 && nonce <= 15 {
                2i8
            } else if nonce >= 16 && nonce <= 30 {
                3i8
            } else {
                4i8
            }
        }
    }

    /// Unit tests.
    #[cfg(not(feature = "ink-experimental-engine"))]
    #[cfg(test)]
    mod tests {
        use super::*;
        // use crate::lixi::*;
        use ink_env::{call, test};
        use ink_lang as ink;
        type Accounts = test::DefaultAccounts<LixiEnv>;

        #[ink::test]
        fn claim_overflow() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            let caller = accounts.alice;
            let contract_id: AccountId = contract_id();
            contract.nonce = 0;
            assert_eq!(contract.lixi(), Err(ContractError::Overflow))
        }

        #[ink::test]
        fn holiday_1_reward_1_should_not_work() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let contract_id: AccountId = contract_id();
            let accounts = default_accounts();
            // Set block to match 1st day
            contract.block = HOLIDAY_1_BLOCK_NUMBER;
            assert_eq!(contract.block, HOLIDAY_1_BLOCK_NUMBER);
            let day_id: DayId = HOLIDAY_1;
            // Set nonce to match reward 1
            // 1=< nonce <= 5
            contract.nonce = 1;
            // update maximum reward 2
            contract
                .reward_per_day
                .insert((day_id, REWARD_TYPE_2), MAX_REWARD_2);
            // update maximum reward 3
            contract
                .reward_per_day
                .insert((day_id, REWARD_TYPE_3), MAX_REWARD_3);
            // TODO: call lixi 1st
            // match case: claim reward 4
            set_balance(accounts.alice, 0);
            contract.lixi();
            let type_of_reward = REWARD_TYPE_4;
            assert_eq!(contract.winners.len(), 1);
            assert_eq!(contract.reward_per_day.len(), 3);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());
            let (_, user_reward) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            let num_of_reward = contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .unwrap();
            // after calling lixi, check balance
            assert_eq!(&get_balance(accounts.alice), user_reward);
            assert_eq!(num_of_reward, &1u32);
            assert_eq!(user_reward, &0u128);
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        #[ink::test]
        fn holiday_1_match_reward_2_should_work() {
            let day_id: DayId = HOLIDAY_1;
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            let caller = accounts.alice;
            let contract_id: AccountId = contract_id();
            // Set block to match 1st day
            contract.block = HOLIDAY_2_BLOCK_NUMBER - 1;
            assert_eq!(contract.block, (HOLIDAY_2_BLOCK_NUMBER - 1));
            // Set nonce to match reward 2
            // 6 =< nonce <= 15
            contract.nonce = 6;
            let reward_type = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_reward = MAX_REWARD_2;

            // TODO: call lixi 1st: success
            // caller: Alice
            contract.lixi();
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO: call lixi 2nd: failure
            // caller: Alice
            assert_eq!(contract.lixi(), Err(ContractError::InvalidRequest));
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &1u32
            );

            // TODO:
            // update maximum quantity
            contract
                .reward_per_day
                .insert((day_id, reward_type), (max_reward - 1));
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &(max_reward - 1)
            );

            // TODO:
            // Call lixi 4th: success
            // caller: Charlie
            set_balance(accounts.charlie, 0);
            set_sender(accounts.charlie);
            contract.lixi();
            let (_, user_reward) = contract.winners.get(&(accounts.charlie, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &max_reward
            );

            // TODO:
            // Call lixi 5th: success
            // match case: reward 3
            set_balance(accounts.eve, 0);
            set_sender(accounts.eve);
            contract.lixi();
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;
            let reward_type = REWARD_TYPE_3;
            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);

            // TODO:
            // update maximum reward 3
            contract
                .reward_per_day
                .insert((day_id, reward_type), max_value);

            set_sender(accounts.frank);
            set_balance(accounts.frank, 0);
            // TODO: call lixi
            contract.lixi();
            let reward_value = REWARD_4_VALUE;
            let reward_type = REWARD_TYPE_4;
            let max_value = MAX_REWARD_4;
            let (_, user_reward) = contract.winners.get(&(accounts.frank, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);
        }

        #[ink::test]
        fn holiday_1_match_reward_3_should_work() {
            let day_id: DayId = HOLIDAY_1;
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            let caller = accounts.alice;
            let contract_id: AccountId = contract_id();
            // Set block to match 1st day
            contract.block = HOLIDAY_2_BLOCK_NUMBER - 1;
            assert_eq!(contract.block, (HOLIDAY_2_BLOCK_NUMBER - 1));

            // Set nonce to match reward 2
            // 16 =< nonce <= 30
            contract.nonce = 16;
            let reward_type = REWARD_TYPE_3;
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;

            set_balance(caller, 0);
            assert_eq!(get_balance(caller), 0);
            assert_eq!(get_balance(contract_id), contract_balance);
            assert!(contract.winners.get(&(caller, day_id)).is_none());

            // TODO: call lixi: 1st
            // caller: Alice
            contract.lixi();
            assert_eq!(contract.winners.len(), 1);
            assert_eq!(contract.reward_per_day.len(), 1);
            assert!(contract
                .reward_per_day
                .get(&(day_id, reward_type))
                .is_some());
            let (_, user_reward_2) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            // after calling lixi, check balance
            assert_eq!(
                get_balance(caller),
                reward_value * 10u128.checked_pow(18).unwrap()
            );
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &1u32
            );

            // TODO: call lixi 3rd: success
            // caller: Bob
            set_balance(accounts.bob, 0);
            set_sender(accounts.bob);
            contract.lixi();
            assert_eq!(
                get_balance(accounts.bob),
                reward_value * 10u128.checked_pow(18).unwrap()
            );
            let (_, user_reward_2) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(user_reward_2, &reward_value);
            assert_eq!(contract.winners.len(), 2);
            assert_eq!(contract.reward_per_day.len(), 1); // only a day = 2i8
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &2u32
            );
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            contract
                .reward_per_day
                .insert((day_id, reward_type), max_value);

            // TODO:
            // update maximum reward 3
            // match case reward 2
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            contract.lixi();
            let reward_type = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);
            assert_eq!(
                contract.reward_per_day.get(&(day_id, reward_type)).unwrap(),
                &1u32
            );
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        #[ink::test]
        pub fn holiday_1_match_reward_4() {
            let day_id: DayId = HOLIDAY_1;
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            let caller = accounts.alice;
            let contract_id: AccountId = contract_id();
            // Set block to match 1st day
            contract.block = HOLIDAY_2_BLOCK_NUMBER - 1;
            assert_eq!(contract.block, (HOLIDAY_2_BLOCK_NUMBER - 1));
            // Set nonce to match reward 4
            // 31 =< nonce <= 100
            contract.nonce = 31;
            let reward_type = REWARD_TYPE_4;
            let reward_value = REWARD_4_VALUE;
            let max_reward = MAX_REWARD_4;

            // TODO: call lixi 1st: success
            // caller: Alice
            contract.lixi();
            let (_, user_reward) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            assert_eq!(user_reward, &0u128);
            assert_eq!(user_reward, &reward_value);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        #[ink::test]
        fn holiday_2_match_reward_1_should_work() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let contract_id: AccountId = contract_id();
            let accounts = default_accounts();
            // Set block to match 1st day
            contract.block = HOLIDAY_2_BLOCK_NUMBER;
            assert_eq!(contract.block, HOLIDAY_2_BLOCK_NUMBER);
            let day_id: DayId = HOLIDAY_2;
            // Set nonce to match reward 1
            // 1=< nonce <= 5
            contract.nonce = 1;
            let type_of_reward = REWARD_TYPE_1;
            let reward_value = REWARD_1_VALUE;
            let max_value = MAX_REWARD_1;

            set_balance(accounts.alice, 0);
            assert_eq!(get_balance(accounts.alice), 0);
            assert_eq!(get_balance(contract_id), contract_balance);
            assert!(contract.winners.get(&(accounts.alice, day_id)).is_none());
            // TODO:
            // Call lixi 1st
            contract.lixi();
            assert_eq!(contract.winners.len(), 1);
            assert_eq!(contract.reward_per_day.len(), 1);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            let num_of_reward = contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .unwrap();
            // after calling lixi, check balance
            assert_eq!(
                get_balance(accounts.alice),
                user_reward * 10u128.checked_pow(18).unwrap()
            );
            assert_eq!(num_of_reward, &1u32);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO:
            // call lixi 2nd
            // match case reward 2
            set_sender(accounts.bob);
            set_balance(accounts.bob, 0);
            contract.lixi();

            let type_of_reward = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_value = MAX_REWARD_2;

            assert_eq!(contract.reward_per_day.len(), 2);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO:
            // update maximum reward 2
            contract
                .reward_per_day
                .insert((day_id, type_of_reward), max_value);
            assert_eq!(
                contract
                    .reward_per_day
                    .get(&(day_id, type_of_reward))
                    .unwrap(),
                &max_value
            );

            // TODO:
            // call lixi 3rd
            // match case reward 3
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            contract.lixi();
            let type_of_reward = REWARD_TYPE_3;
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;

            assert_eq!(contract.reward_per_day.len(), 3);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(user_reward, &reward_value);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        /// Creates a new instance of `GiveMe` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.

        fn create_contracts(initial_balance: Balance) -> LixiApp {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            LixiApp::new()
        }

        fn contract_id() -> AccountId {
            ink_env::test::get_current_contract_account_id::<LixiEnv>()
                .expect("Cannot get contract id")
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<LixiEnv>();
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            )
        }

        fn default_accounts() -> Accounts {
            test::default_accounts().expect("Test environment is expected to be initialized.")
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account_id, balance)
                .expect("Cannot set account balance");
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        fn build_contract() -> LixiApp {
            let accounts = default_accounts();
            LixiApp::new()
        }
    }
}
