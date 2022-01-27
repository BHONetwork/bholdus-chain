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
    use ink_prelude::vec;
    use ink_prelude::vec::Vec;
    use ink_storage::collections::HashMap as StorageHashMap;
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
    pub const MAX_REWARD_2: Quantity = 68;
    pub const MAX_REWARD_3: Quantity = 333;
    pub const MAX_REWARD_4: Quantity = u32::MAX;

    #[ink(storage)]
    pub struct LixiApp {
        // use for testing, set blocknumber
        pub block: BlockNumber,
        // use for randomness algorithm
        pub nonce: u8,
        // (AccountId, DayId)
        pub winners: StorageHashMap<(AccountId, DayId), (Timestamp, Balance)>,
        // (DayId, RewardType)
        pub reward_per_day: StorageHashMap<(DayId, RewardType), Quantity>,
        pub users: StorageHashMap<AccountId, Vec<(Timestamp, Balance)>>,
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
                users: StorageHashMap::default(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Lixi App
        #[ink(message)]
        pub fn lixi(&mut self, to: AccountId) -> Result<Balance, ContractError> {
            let account_id: AccountId = self.env().account_id();
            let timestamp: Timestamp = self.env().block_timestamp();

            //#[test]
            let block_number: BlockNumber = self.block;
            // let div = self.nonce % 100;

            let subject_runtime: [u8; 32] = [self.nonce; 32];
            let random_seed = self.env().extension().fetch_random(subject_runtime)?;
            let random: &[u8] = &random_seed;
            let random_vec: Vec<u8> = random.to_vec();
            let index0 = random_vec[0];
            // let block_number: BlockNumber = self.env().block_number();
            // div in range: [1..100];
            let div = index0 % 100;

            let mapping_reward = if div >= 1 && div <= 5 {
                REWARD_TYPE_1
            } else if div >= 6 && div <= 15 {
                REWARD_TYPE_2
            } else if div >= 16 && div <= 30 {
                REWARD_TYPE_3
            } else if div >= 31 && div <= 100 {
                REWARD_TYPE_4
            } else {
                REWARD_TYPE_4
            };

            let holiday = if block_number >= HOLIDAY_1_BLOCK_NUMBER
                && block_number < HOLIDAY_2_BLOCK_NUMBER
            {
                HOLIDAY_1
            } else if block_number >= HOLIDAY_2_BLOCK_NUMBER
                && block_number < HOLIDAY_3_BLOCK_NUMBER
            {
                HOLIDAY_2
            } else if block_number >= HOLIDAY_3_BLOCK_NUMBER && block_number < END_OF_HOLIDAY {
                HOLIDAY_3
            } else {
                return Err(ContractError::Overflow);
            };

            let user = self.winners.get(&(to, holiday));
            if user.is_some() {
                return Err(ContractError::InvalidRequest);
            };
            let matched_reward = self.recursion(mapping_reward, holiday);
            let (amount, (type_of_reward, num_of_rewards)) = match matched_reward {
                (reward_type, value_of_reward, curr_quantity) => {
                    (value_of_reward, (reward_type, curr_quantity + 1))
                }
                _ => {
                    return Err(ContractError::Overflow);
                }
            };
            let actual_reward: Balance = amount * 10u128.checked_pow(18).unwrap();
            // TODO: Do transfer
            self.give_me(to, actual_reward);
            self.insert_nonce(self.nonce);
            self.winners
                .insert((to, holiday), (timestamp, actual_reward));

            let mut user_reward: Vec<(Timestamp, Balance)> =
                self.users.get(&to).clone().unwrap_or(&vec![]).to_vec();
            user_reward.push((timestamp, actual_reward));
            self.users.insert(to, user_reward.to_vec());

            self.reward_per_day
                .insert((holiday, type_of_reward), num_of_rewards);

            self.env().emit_event(Reward {
                from: account_id,
                to,
                value: actual_reward,
                timestamp,
            });
            Ok(amount)
        }

        /// transfer from funds
        #[inline]
        pub fn give_me(&mut self, to: AccountId, value: Balance) {
            assert!(value <= self.env().balance(), "insufficient funds!");

            if self.env().transfer(to, value).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds."
                )
            }
        }

        #[ink(message)]
        pub fn get_winners(&self, owner: AccountId) -> Vec<(Timestamp, Balance)> {
            self.users.get(&owner).clone().unwrap_or(&vec![]).to_vec()
        }

        /// Check limit
        #[ink(message)]
        pub fn is_limit(&self, to: AccountId) -> Result<bool, ContractError> {
            // let block_number = self.env().block_number();
            let block_number: BlockNumber = self.block;
            let holiday = if block_number >= HOLIDAY_1_BLOCK_NUMBER
                && block_number < HOLIDAY_2_BLOCK_NUMBER
            {
                HOLIDAY_1
            } else if block_number >= HOLIDAY_2_BLOCK_NUMBER
                && block_number < HOLIDAY_3_BLOCK_NUMBER
            {
                HOLIDAY_2
            } else if block_number >= HOLIDAY_3_BLOCK_NUMBER && block_number < END_OF_HOLIDAY {
                HOLIDAY_3
            } else {
                return Err(ContractError::Overflow);
            };

            let user = self.winners.get(&(to, holiday));
            Ok(user.is_some())
        }

        /// Returns balance of smart contract.
        #[ink(message)]
        pub fn balance_of(&self) -> Balance {
            self.env().balance()
        }

        /// AccountId of smart contract
        #[ink(message)]
        pub fn account_id(&self) -> AccountId {
            self.env().account_id()
        }

        #[inline]
        fn insert_nonce(&mut self, nonce: u8) {
            if nonce == u8::MAX {
                self.nonce = 0
            } else {
                self.nonce += 1
            }
        }

        #[inline]
        pub fn recursion(
            &self,
            type_of_reward: RewardType,
            day_id: DayId,
        ) -> (RewardType, Balance, &Quantity) {
            match type_of_reward {
                1 => {
                    if day_id == HOLIDAY_1 {
                        // unavailable reward 1 in 1st day
                        self.work_with_reward_2(day_id)
                    } else if day_id == HOLIDAY_2 {
                        self.work_with_reward_1(day_id)
                    } else {
                        // day_id == HOLIDAY_3
                        // ensure no user claim REWARD_TYPE_3 in 2nd day
                        let is_existed_reward_1 = self
                            .reward_per_day
                            .contains_key(&(HOLIDAY_2, REWARD_TYPE_1));
                        if is_existed_reward_1 {
                            // unavailable REWARD_TYPE_1
                            self.work_with_reward_2(day_id)
                        } else {
                            // available REWARD_TYPE_1
                            self.work_with_reward_1(day_id)
                        }
                    }
                }
                2 => self.work_with_reward_2(day_id),
                3 => self.work_with_reward_3(day_id),
                // (4, value, max_quantity)
                _ => {
                    let quantity = self
                        .reward_per_day
                        .get(&(day_id, REWARD_TYPE_4))
                        .unwrap_or(&0u32);
                    (REWARD_TYPE_4, REWARD_4_VALUE, quantity)
                }
            }
        }

        #[inline]
        pub fn work_with_reward_1(&self, day_id: DayId) -> (RewardType, Balance, &Quantity) {
            let curr_quantity = self
                .reward_per_day
                .get(&(day_id, REWARD_TYPE_1))
                .unwrap_or(&0u32);
            if curr_quantity == &MAX_REWARD_1 {
                // unavailable REWARD_TYPE_1
                self.work_with_reward_2(day_id)
            } else {
                // available REWARD_TYPE_1
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
                // Available REWARD_TYPE_3
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

        /// Only support to test. Set blocknumber.
        #[ink(message)]
        pub fn set_block_to_test(&mut self, block: BlockNumber) -> Result<(), ContractError> {
            self.block = block;
            Ok(())
        }

        /// Only support to test. Query blocknumber
        #[ink(message)]
        pub fn get_block_to_test(&self) -> BlockNumber {
            self.block
        }

        /// Only support to test. Check nonce
        #[ink(message)]
        pub fn get_nonce(&self) -> u8 {
            self.nonce
        }

        /// Only support to test. Reset reward of user.
        #[ink(message)]
        pub fn reset(&mut self, user: AccountId) -> Result<(), ContractError> {
            let ids: Vec<DayId> = vec![1, 2, 3];
            for day_id in ids.iter() {
                self.winners.take(&(user, *day_id));
            }
            self.users.take(&user);

            Ok(())
        }
        /// Only support to test. Set max nonce.
        #[ink(message)]
        pub fn set_max_nonce(&mut self) -> Result<(), ContractError> {
            self.nonce = u8::MAX;
            Ok(())
        }
    }

    /// Unit tests.
    #[cfg(not(feature = "ink-experimental-engine"))]
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::{call, test};
        use ink_lang as ink;
        type Accounts = test::DefaultAccounts<LixiEnv>;

        #[ink::test]
        fn claim_overflow() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            contract.nonce = 0;
            let caller = accounts.alice;
            // self.block == 0
            assert_eq!(contract.lixi(caller), Err(ContractError::Overflow))
        }

        #[ink::test]
        fn blocknumber_overflow() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let accounts = default_accounts();
            let caller = accounts.alice;
            contract.block = END_OF_HOLIDAY + 1;
            // self.block == 0
            assert_eq!(contract.lixi(caller), Err(ContractError::Overflow))
        }

        #[ink::test]
        fn limit_should_work() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let contract_id: AccountId = contract_id();
            let accounts = default_accounts();
            // Set block to match 1st day
            contract.block = HOLIDAY_3_BLOCK_NUMBER;
            assert_eq!(contract.block, HOLIDAY_3_BLOCK_NUMBER);
            let day_id: DayId = HOLIDAY_1;
            // Set nonce to match reward 2
            // 6 =< nonce <= 15
            contract.nonce = 6;
            let type_of_reward = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_reward = MAX_REWARD_2;
            let actual_reward_value = to_balance(reward_value);

            // TODO: caller: accounts.bob
            // call lixi x1: success
            contract.lixi(accounts.bob);
            let contract_balance = contract_balance - actual_reward_value;
            assert_eq!(contract.winners.len(), 1);
            assert_eq!(contract.reward_per_day.len(), 1);
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO: caller: accounts.bob
            // call lixi x2: failure
            assert_eq!(
                contract.lixi(accounts.bob),
                Err(ContractError::InvalidRequest)
            );

            // TODO: check limit
            assert_eq!(contract.is_limit(accounts.bob), Ok(true));
            assert_eq!(contract.is_limit(accounts.alice), Ok(false));
            // Set block: day 1
            contract.block = HOLIDAY_1_BLOCK_NUMBER;
            assert_eq!(contract.is_limit(accounts.bob), Ok(false));

            // Set block: day 2
            contract.block = HOLIDAY_2_BLOCK_NUMBER;
            contract.lixi(accounts.eve);
            let contract_balance = contract_balance - actual_reward_value;
            assert_eq!(contract.winners.len(), 2);
            assert_eq!(contract.reward_per_day.len(), 2);
            assert_eq!(get_balance(contract_id), contract_balance);

            assert_eq!(contract.is_limit(accounts.bob), Ok(false));
            assert_eq!(contract.is_limit(accounts.eve), Ok(true));
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
            let caller = accounts.alice;
            contract.lixi(caller);
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
            let caller = accounts.alice;
            contract.lixi(caller);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO: call lixi 2nd: failure
            // caller: Alice
            assert_eq!(contract.lixi(caller), Err(ContractError::InvalidRequest));
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
            // set_sender(accounts.charlie);
            let caller = accounts.charlie;
            contract.lixi(caller);
            let (_, user_reward) = contract.winners.get(&(accounts.charlie, day_id)).unwrap();
            assert_eq!(
                user_reward,
                &(reward_value * 10u128.checked_pow(18).unwrap())
            );
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
            // set_sender(accounts.eve);
            let caller = accounts.eve;
            contract.lixi(caller);
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;
            let reward_type = REWARD_TYPE_3;
            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(
                user_reward,
                &(reward_value * 10u128.checked_pow(18).unwrap())
            );

            // TODO:
            // update maximum reward 3
            contract
                .reward_per_day
                .insert((day_id, reward_type), max_value);

            // set_sender(accounts.frank);
            let caller = accounts.frank;
            set_balance(accounts.frank, 0);
            // TODO: call lixi
            contract.lixi(caller);
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
            let caller = accounts.alice;
            contract.lixi(caller);
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
            // set_sender(accounts.bob);
            let caller = accounts.bob;
            contract.lixi(caller);
            assert_eq!(
                get_balance(accounts.bob),
                reward_value * 10u128.checked_pow(18).unwrap()
            );
            let (_, user_reward_2) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(
                user_reward_2,
                &(reward_value * 10u128.checked_pow(18).unwrap())
            );
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
            // set_sender(accounts.eve);
            let caller = accounts.eve;
            set_balance(accounts.eve, 0);
            contract.lixi(caller);
            let reward_type = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(
                user_reward,
                &(reward_value * 10u128.checked_pow(18).unwrap())
            );
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
            contract.nonce = u8::MAX;
            let reward_type = REWARD_TYPE_4;
            let reward_value = REWARD_4_VALUE;
            let max_reward = MAX_REWARD_4;

            // TODO: call lixi 1st: success
            // caller: Alice
            let caller = accounts.alice;
            contract.lixi(caller);
            let (_, user_reward) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            assert_eq!(user_reward, &0u128);
            assert_eq!(user_reward, &reward_value);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO: call lixi 2nd
            // caller: bob
            let caller = accounts.bob;
            contract.lixi(caller);
            assert_eq!(contract.nonce, 1);
            let reward_type = REWARD_TYPE_4;
            let reward_value = REWARD_4_VALUE;
            let max_reward = MAX_REWARD_4;
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
            let caller = accounts.alice;
            contract.lixi(caller);
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
            assert_eq!(&get_balance(accounts.alice), user_reward);
            assert_eq!(num_of_reward, &1u32);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO:
            // call lixi 2nd
            // match case reward 2
            // set_sender(accounts.bob);
            let caller = accounts.bob;
            set_balance(accounts.bob, 0);
            contract.lixi(caller);

            let type_of_reward = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_value = MAX_REWARD_2;

            assert_eq!(contract.reward_per_day.len(), 2);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(
                user_reward / (10u128.checked_pow(18).unwrap()),
                reward_value
            );
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
            // set_sender(accounts.eve);
            let caller = accounts.eve;
            set_balance(accounts.eve, 0);
            contract.lixi(caller);
            let type_of_reward = REWARD_TYPE_3;
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;

            assert_eq!(contract.reward_per_day.len(), 3);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(
                user_reward / (10u128.checked_pow(18).unwrap()),
                reward_value
            );
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        #[ink::test]
        fn holiday_3_match_reward_1_should_work() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let contract_id: AccountId = contract_id();
            let accounts = default_accounts();
            // Set block to match 3nd day
            contract.block = HOLIDAY_3_BLOCK_NUMBER;
            assert_eq!(contract.block, HOLIDAY_3_BLOCK_NUMBER);
            let day_id: DayId = HOLIDAY_3;
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
            let caller = accounts.alice;
            contract.lixi(caller);
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
            assert_eq!(&get_balance(accounts.alice), user_reward);
            assert_eq!(num_of_reward, &1u32);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO:
            // call lixi 2nd
            // match case reward 2
            // set_sender(accounts.bob);
            let caller = accounts.bob;
            set_balance(accounts.bob, 0);
            contract.lixi(caller);

            let type_of_reward = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_value = MAX_REWARD_2;

            assert_eq!(contract.reward_per_day.len(), 2);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(
                user_reward / (10u128.checked_pow(18).unwrap()),
                reward_value
            );
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
            // set_sender(accounts.eve);
            let caller = accounts.eve;
            set_balance(accounts.eve, 0);
            contract.lixi(caller);
            let type_of_reward = REWARD_TYPE_3;
            let reward_value = REWARD_3_VALUE;
            let max_value = MAX_REWARD_3;

            assert_eq!(contract.reward_per_day.len(), 3);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.eve, day_id)).unwrap();
            assert_eq!(
                user_reward / (10u128.checked_pow(18).unwrap()),
                reward_value
            );
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);
        }

        #[ink::test]
        pub fn holiday_3_match_reward_1_should_not_work() {
            let contract_balance = 1000000 * 10u128.checked_pow(18).unwrap();
            let mut contract = create_contracts(contract_balance);
            let contract_id: AccountId = contract_id();
            let accounts = default_accounts();
            set_balance(accounts.alice, 0);

            //Set block to match 2nd day
            //

            contract.block = HOLIDAY_2_BLOCK_NUMBER;
            assert_eq!(contract.block, HOLIDAY_2_BLOCK_NUMBER);
            let day_id: DayId = HOLIDAY_2;
            // Set nonce to match reward 1
            // 1=< nonce <= 5
            contract.nonce = 2;
            let type_of_reward = REWARD_TYPE_1;
            let reward_value = REWARD_1_VALUE;
            let max_value = MAX_REWARD_1;

            // TODO:
            // Call lixi 1st
            let caller = accounts.alice;
            contract.lixi(caller);
            let num_of_reward = contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .unwrap();
            assert_eq!(&contract.reward_per_day.len(), num_of_reward);
            assert_eq!(contract.winners.len(), 1);
            let (_, user_reward) = contract.winners.get(&(accounts.alice, day_id)).unwrap();
            assert_eq!(&get_balance(accounts.alice), user_reward);
            let contract_balance =
                contract_balance - reward_value * 10u128.checked_pow(18).unwrap();
            assert_eq!(get_balance(contract_id), contract_balance);

            // TODO:
            // Call lixi
            // Set nonce to match reward 1
            // 1=< nonce <= 5
            contract.nonce = 3;
            // Set block to match 3rd day
            contract.block = HOLIDAY_3_BLOCK_NUMBER + 1;
            assert_eq!(contract.block, HOLIDAY_3_BLOCK_NUMBER + 1);
            let day_id: DayId = HOLIDAY_3;

            // set_sender(accounts.bob);
            let caller = accounts.bob;
            set_balance(accounts.bob, 0);
            contract.lixi(caller);
            // reward_1  existed in 2nd, match case reward 2
            let type_of_reward = REWARD_TYPE_2;
            let reward_value = REWARD_2_VALUE;
            let max_value = MAX_REWARD_2;

            assert_eq!(contract.reward_per_day.len(), 2);
            assert!(contract
                .reward_per_day
                .get(&(day_id, type_of_reward))
                .is_some());

            let (_, user_reward) = contract.winners.get(&(accounts.bob, day_id)).unwrap();
            assert_eq!(
                user_reward / (10u128.checked_pow(18).unwrap()),
                reward_value
            );
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

        fn to_balance(value: Balance) -> Balance {
            value * 10u128.checked_pow(18).unwrap()
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
