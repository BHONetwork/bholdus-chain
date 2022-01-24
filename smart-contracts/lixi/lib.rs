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
    pub const DAY_I: DayId = 1;
    pub const DAY_II: DayId = 2;
    pub const DAY_III: DayId = 3;
    pub const HOLIDAY_1: BlockNumber = 1083296;
    pub const HOLIDAY_2: BlockNumber = 1112096;
    pub const HOLIDAY_3: BlockNumber = 1140896;
    pub const END_OF_HOLIDAY: BlockNumber = 1169696;
    pub const REWARD_1_VALUE: Balance = 99999;
    pub const REWARD_2_VALUE: Balance = 2000;
    pub const REWARD_3_VALUE: Balance = 500;
    pub const REWARD_4_VALUE: Balance = 0;
    pub const MAX_REWARD_1: u32 = 1;
    pub const MAX_REWARD_2: u32 = 204;
    pub const MAX_REWARD_3: u32 = 333;
    const WRONG_DAY: &str = "Invalid transaction. Abort";

    #[ink(storage)]
    pub struct LixiApp {
        pub nonce: u32,
        randomness: Vec<u32>,
        vec: Vec<(u8, u8, u8)>,
        random_seed: [u8; 32],
        balance: Balance,
        pub winners: StorageHashMap<(AccountId, DayId), (Timestamp, Balance)>,
        div: Vec<u8>,
        pub reward_per_day: StorageHashMap<(DayId, i8), u32>,
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

    #[ink(event)]
    pub struct Claimed {
        #[ink(topic)]
        user: AccountId,
        value: Balance,
        timestamp: Timestamp,
    }

    #[derive(scale::Encode, scale::Decode, Clone, Copy, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub enum RewardType {
        One,
        Two,
        Three,
        Four,
    }

    impl LixiApp {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                nonce: Default::default(),
                randomness: Default::default(),
                random_seed: Default::default(),
                balance: Default::default(),
                winners: StorageHashMap::default(),
                reward_per_day: StorageHashMap::default(),
                vec: Default::default(),
                div: Default::default(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Panic if the AccountId-DayId does not exit.
        fn ensure_no_winner(&self, account_id: AccountId, day_id: DayId) {
            assert!(!self.winners.contains_key(&(account_id, day_id)));
        }

        /// Ensure available reward per day
        fn ensure_available_reward(&self, quantity: &u32, max_quantity: &u32) {
            assert!(quantity < max_quantity);
        }

        /// Lixi App
        #[ink(message)]
        pub fn lixi(&mut self, caller: AccountId) -> Result<Balance, ContractError> {
            // let caller: AccountId = self.env().caller();
            let account_id: AccountId = self.env().account_id();
            let block_number: BlockNumber = self.env().block_number();
            let timestamp: Timestamp = self.env().block_timestamp();

            let subject_runtime: [u32; 32] = [self.nonce; 32];
            let subject: u32 = self.nonce;

            // #[ink::test]
            // let (_hash, random_block) = self.env().random(&[subject]);

            // let random_seed = self.env().extension().fetch_random(subject_runtime)?;
            // let random: &[u8] = &random_seed;
            // let random_vec: Vec<u8> = random.to_vec();
            // let index0 = random_vec[0];
            // let div = index0 % 100;
            let div = subject % 100;
            ink_env::debug_println!("ink_lixi div: {:?}; subject {:?}", div, subject);
            let reward_type = if div >= 1 && div <= 5 {
                (1, REWARD_1_VALUE, MAX_REWARD_1)
            } else if div >= 6 && div <= 15 {
                (2, REWARD_2_VALUE, MAX_REWARD_2)
            } else if div >= 16 && div <= 30 {
                (3, REWARD_3_VALUE, MAX_REWARD_3)
            } else if div >= 31 && div <= 100 {
                (4, REWARD_4_VALUE, 0)
            } else {
                (0, 0, 0)
            };

            let (amount, day_id, (type_of_reward, num_of_rewards)) = if
            // block_number >= HOLIDAY_1
            // &&
            block_number < HOLIDAY_2 {
                // Action: M1
                let holiday: DayId = DAY_I;
                ink_env::debug_println!("ink_lixi day: {:?}", &holiday);
                let user = self.winners.get(&(caller, holiday));
                ink_env::debug_println!("ink_lixi winners: {:?}", &user);
                if user.is_some() {
                    ink_env::debug_println!("ink_lixi existed");
                    return Err(ContractError::InvalidRequest);
                };
                ink_env::debug_println!("ink_lixi match reward_type: {:?}", &reward_type);
                //self.ensure_no_winner(caller, holiday);
                match reward_type {
                    (4, value, _) => {
                        // Don't need check quantity
                        let reward_type: i8 = 4i8;
                        let quantity = self
                            .reward_per_day
                            .get(&(holiday, reward_type))
                            .unwrap_or(&0u32);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    (reward_type, value, max_quantity) => {
                        let quantity = self
                            .reward_per_day
                            .get(&(holiday, reward_type))
                            .unwrap_or(&0u32);
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    _ => (0, 0, (0, 0)),
                }
            } else if block_number >= HOLIDAY_2 && block_number < HOLIDAY_3 {
                // Action: M2
                let holiday: DayId = DAY_II;
                ink_env::debug_println!("ink_lixi day: {:?}", &holiday);
                let user = self.winners.get(&(caller, holiday));
                ink_env::debug_println!("ink_lixi winners: {:?}", &user);
                if user.is_some() {
                    ink_env::debug_println!("ink_lixi existed");
                    return Err(ContractError::InvalidRequest);
                };
                match reward_type {
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
                        assert!(!self.reward_per_day.contains_key(&(DAY_I, reward_type)));
                        let quantity = self
                            .reward_per_day
                            .get(&(holiday, reward_type))
                            .unwrap_or(&0u32);
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    (reward_type, value, max_quantity) => {
                        let quantity = self
                            .reward_per_day
                            .get(&(holiday, reward_type))
                            .unwrap_or(&0u32);
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    // Impossible cases
                    _ => (0, 0, (0, 0)),
                }
            } else if block_number >= HOLIDAY_3 && block_number < END_OF_HOLIDAY {
                let holiday: DayId = DAY_III;
                ink_env::debug_println!("ink_lixi day: {:?}", &holiday);
                let user = self.winners.get(&(caller, holiday));
                ink_env::debug_println!("ink_lixi winners: {:?}", &user);
                if user.is_some() {
                    ink_env::debug_println!("ink_lixi existed");
                    return Err(ContractError::InvalidRequest);
                };
                self.ensure_no_winner(caller, holiday);
                match reward_type {
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
                        assert!(!self.reward_per_day.contains_key(&(DAY_I, reward_type)));
                        assert!(!self.reward_per_day.contains_key(&(DAY_II, reward_type)));
                        let quantity = self.reward_per_day.get(&(holiday, reward_type)).unwrap();
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    (reward_type, value, max_quantity) => {
                        let quantity = self.reward_per_day.get(&(holiday, reward_type)).unwrap();
                        self.ensure_available_reward(quantity, &max_quantity);
                        (value, holiday, (reward_type, quantity + 1))
                    }
                    _ => (0, 0, (0, 0)),
                }
            } else {
                (0, 0, (0, 0))
            };

            ink_env::debug_println!(
                "ink_lixi caller{:?}; amount {:?}; day_id {:?}",
                &caller,
                &amount,
                &day_id
            );
            let reward: Balance = amount * 10u128.checked_pow(18).unwrap();

            // TODO: Do transfer
            self.give_me(reward)?;
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

        #[ink(message)]
        pub fn get_vec(&self) -> Vec<(u8, u8, u8)> {
            self.vec.clone()
        }

        #[ink(message)]
        pub fn get_div(&self) -> Vec<u8> {
            self.div.clone()
        }

        #[ink(message)]
        pub fn get_nonce(&self) -> u32 {
            self.nonce
        }

        #[ink(message)]
        pub fn get_rand(&self) -> Vec<u32> {
            self.randomness.clone()
        }

        #[ink(message)]
        pub fn get_random_seed(&self) -> [u8; 32] {
            self.random_seed
        }

        /*/// Get list rewards
        #[ink(message)]
        pub fn get_rewards(&self, owner: AccountId) -> Option<Balance> {
        self.rewards.get(&owner).cloned()
        }*/

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

        /// Give amount token to caller
        #[inline]
        pub fn give_me(&mut self, value: Balance) -> Result<(), ContractError> {
            self.env().transfer(self.env().caller(), value);
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lixi::REWARD_1_VALUE;
    use ink_env::{call, test};
    use ink_lang as ink;
    type Accounts = test::DefaultAccounts<LixiEnv>;

    fn default_accounts() -> Accounts {
        test::default_accounts().expect("Test environment is expected to be initialized.")
    }

    fn build_contract() -> LixiApp {
        let accounts = default_accounts();
        LixiApp::new()
    }

    #[ink::test]
    fn test_lixi() {
        let mut contract = build_contract();
        let accounts = default_accounts();
        let caller = accounts.alice;
        // Case 1:
        contract.nonce = 1;
        let day_id = 1i8;
        let reward_type = 1i8;
        let reward_value = REWARD_1_VALUE;
        assert!(contract.winners.get(&(caller, day_id)).is_none());
        contract.lixi(caller);
        assert_eq!(contract.winners.len(), 1);
        assert_eq!(contract.reward_per_day.len(), 1);

        assert!(contract
            .reward_per_day
            .get(&(day_id, reward_type))
            .is_some());
        assert!(contract.winners.get(&(caller, day_id)).is_some());
    }
}
