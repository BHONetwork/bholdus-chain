#![cfg_attr(not(feature = "std"), no_std)]
use core::convert::TryInto;
use ink_env::{AccountId, Environment, Error, Hash};
use ink_lang as ink;
use ink_prelude::vec;
use ink_prelude::vec::Vec;
use ink_storage::{lazy::Mapping, Lazy};
type DayId = i8;

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
mod lixi {
    use super::*;
    use ink_storage::{
        collections::{hashmap::Entry, HashMap as StorageHashMap, Vec as StorageVec},
        traits::{PackedLayout, SpreadLayout},
        Lazy,
    };
    #[ink(storage)]
    pub struct LixiApp {
        nonce: u8,
        randomness: Vec<u32>,
        balance: Balance,
        winners: StorageHashMap<(AccountId, DayId), (Timestamp, Balance)>,
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

    impl LixiApp {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                nonce: Default::default(),
                randomness: Default::default(),
                balance: Default::default(),
                winners: Default::default(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Lixi App
        #[ink(message)]
        pub fn lixi(&mut self) -> Result<Balance, ContractError> {
            let caller: AccountId = self.env().caller();
            let account_id: AccountId = self.env().account_id();
            let block_number: BlockNumber = self.env().block_number();
            let timestamp: Timestamp = self.env().block_timestamp();
            let subject_runtime: [u8; 32] = [self.nonce; 32];
            let random_seed = self.env().extension().fetch_random(subject_runtime)?;
            let (_hash, randomness) = self.env().random(&random_seed);
            let lucky_number = randomness % 3;
            let amount: Balance = if lucky_number == 0 {
                10
            } else if lucky_number == 1 {
                20
            } else {
                30
            };
            let reward: Balance = amount * 10u128.checked_pow(18).unwrap();
            // TODO: Do transfer
            self.give_me(reward);
            self.winners.insert((caller, 1i8), (timestamp, amount));

            // let rewards = &mut self.rewards;
            // rewards.push(amount);

            // ink_env::debug_println!("ink_rewards {:?}", &rewards);
            // self.rewards = rewards.to_vec();

            self.nonce += 1;
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
        pub fn get_nonce(&self) -> u8 {
            self.nonce
        }

        #[ink(message)]
        pub fn get_rand(&self) -> Vec<u32> {
            self.randomness.clone()
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
