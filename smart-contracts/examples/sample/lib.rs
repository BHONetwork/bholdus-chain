#![cfg_attr(not(feature = "std"), no_std)]
#![feature(int_abs_diff)]
use ink_env::{AccountId, Environment, Error};
use ink_lang as ink;
use ink_prelude::vec;
use ink_prelude::vec::Vec;

#[ink::chain_extension]
pub trait ChainExtension {
    type ErrorCode = ContractError;
    // Use the #[ink(extension = {func_id})] syntax to specify the function id.
    // We will `match` on this in the runtime to map this to some custom pallet extrinsic
    #[ink(extension = 1)]
    /// Calls the runtime chain extension with func_id 1, defined in the runtime, which receives a
    /// number and stores it in a runtime storage value
    fn do_store_in_runtime(key: u32) -> Result<u32, ContractError>;
    #[ink(extension = 2)]
    /// Calls the runtime chain extension with func_id 2, which uses pallet_balances::transfer to
    /// perform a transfer of `value` from the sender, to `recipient`
    fn do_balance_transfer(
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
        recipient: AccountId,
    ) -> Result<u32, ContractError>;

    #[ink(extension = 3)]
    /// Calls the runtime chain extension with func_id 3, which calls free_balance of
    /// pallet_balances for the given account.
    fn do_get_balance(account: AccountId) -> Result<u32, ContractError>;

    #[ink(extension = 4, returns_result = false)]
    /// Calls the runtime chain extension with func_id 4, to get the current value held in the
    /// runtime storage value.
    fn do_get_from_runtime() -> u32;

    #[ink(extension = 5)]
    /// Calls the runtime chain extension with func_id 3, which calls free_balance of
    /// pallet_balances for the given account.
    fn claim(
        from: AccountId,
        recipient: AccountId,
        value: <ink_env::DefaultEnvironment as Environment>::Balance,
    ) -> Result<(), ContractError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    FailToCallRuntime,
    UnknownStatusCode,
    InvalidScaleEncoding,
    InsufficientBalance,
}

impl From<scale::Error> for ContractError {
    fn from(_: scale::Error) -> Self {
        ContractError::InvalidScaleEncoding
    }
}

// Define error codes here for error situations not sufficiently captured in `Result`s returned from
// runtime. Then, return error code from runtime chain extension using Ok(RetVal::Converging({your
// error code}))
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
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    type ChainExtension = ChainExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
/// A smart contract with a custom environment, necessary for the chain extension
mod contract_with_extension {
    use super::*;
    use core::convert::TryInto;
    /// Defines the storage of our contract.
    #[ink(storage)]
    pub struct RuntimeInterface {
        stored_number: u32,
    }

    #[ink(event)]
    pub struct ResultNum {
        number: u32,
    }

    // impl for smart contract functions that demonstrate two way communication between runtime and
    // smart contract
    impl RuntimeInterface {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                stored_number: Default::default(),
            }
        }

        /// Get currently stored value
        #[ink(message)]
        pub fn get_value(&mut self) -> u32 {
            self.env().emit_event(ResultNum {
                number: self.stored_number.clone(),
            });
            self.stored_number
        }

        /// Simply returns the current value.
        #[ink(message)]
        pub fn get(&self) -> Balance {
            // ink_env::balance::<CustomEnvironment>()
            // self.stored_number
            self.env().balance()
        }

        /// Returns a random hash seed
        #[ink(message)]
        // pub fn random(&self, subject: u8) -> Result<(Hash, BlockNumber), ContractError> {
        //     let (hash, block) = self.env().random(&[subject]);
        //     Ok((hash, block))
        // }
        pub fn randomess(&self, subject: u8) -> (Hash, BlockNumber) {
            let (hash, block_number) = self.env().random(&[subject]);
            (hash, block_number)
        }

        /// Returns a random hash seed
        #[ink(message)]
        pub fn random_m(&self, value: u8) -> BlockNumber {
            // let block_number = self.get_block_number();
            let subject: Vec<u8> = (0..7).map(|v| v + value).collect();
            let (hash, n0) = self.env().random(&subject);
            n0
        }

        #[ink(message)]
        pub fn lixi(&self, lucky_number: u8, select: u8) -> (u8, u8, u64, Balance) {
            let bounded_vec: Vec<u8> = (0u8..100u8).map(|v| v).collect();
            let bounded_lucky_number = if bounded_vec.contains(&lucky_number) {
                lucky_number
            } else {
                0
            };
            let bounded_select: u8 = if select == 0 | 1 | 2 { select } else { 0 };
            // let block_number = self.get_block_number();
            let now: u64 = self.get_timestamp();
            let s0: Vec<u8> = (0..7).map(|v| v + bounded_lucky_number).collect(); // 7 days + lucky_number
            let (_, r0) = self.env().random(&[bounded_lucky_number]);
            let (_, r1) = self.env().random(&s0);
            let number: u64 = u32::from(r0 + r1).into();
            let n = (number + now) % 3;
            let balance = if n == bounded_select.into() {
                /* let sub: u32 = r0.abs_diff(r1);
                let u8sub: u8 = sub.try_into().unwrap();
                let l = if bounded_vec.contains(&u8sub) {
                    sub
                } else {
                    bounded_lucky_number.try_into().unwrap()
                };
                // error trapped
                // let (_, now_r) = self.env().random(&[now.try_into().unwrap()]);
                let (_, rr) = self.env().random(&[l.try_into().unwrap()]);
                */
                let div: u32 = r1 % 3;
                if div == 0 {
                    100
                } else if div == 1 {
                    1000
                } else {
                    assert_eq!(div, 2);
                    10000
                }
            } else {
                let div: u32 = r1 % 3;
                if div == 0 {
                    10
                } else if div == 1 {
                    20
                } else {
                    assert_eq!(div, 2);
                    30
                }
            };
            (bounded_select, bounded_lucky_number, n, balance)
        }

        /// Returns the current block number
        // #[inline]
        #[ink(message)]
        pub fn get_block_number(&self) -> BlockNumber {
            self.env().block_number()
        }

        // #[inline]
        #[ink(message)]
        pub fn get_timestamp(&self) -> Timestamp {
            self.env().block_timestamp()
        }

        /// Returns the account ID of the executed contract
        #[ink(message)]
        pub fn smart_contract_account(&self) -> AccountId {
            // ink_env::account_id::<CustomEnvironment>()
            self.env().account_id()
        }

        #[ink(message)]
        pub fn caller_account(&mut self) -> AccountId {
            self.env().caller()
        }

        /// A simple storage function meant to demonstrate calling a smart contract from a custom
        /// pallet. Here, we've set the `selector` explicitly to make it simpler to target this
        /// function.
        #[ink(message, selector = 0xABCDEF)]
        pub fn set_value(&mut self, value: u32) -> Result<(), ContractError> {
            self.stored_number = value;
            self.env().emit_event(ResultNum { number: value });
            Ok(())
        }

        /// Invoke the extended custom pallet extrinsic with the argument given to the smart
        /// contract function
        #[ink(message)]
        pub fn store_in_runtime(&mut self, value: u32) -> Result<(), ContractError> {
            self.env().extension().do_store_in_runtime(value)?;
            self.env().emit_event(ResultNum { number: value });
            self.stored_number = value;
            Ok(())
        }

        // Invoke the extended transfer function with the arguments given to the smart contract
        // function
        #[ink(message)]
        pub fn extended_transfer(
            &mut self,
            amount: Balance,
            recipient: AccountId,
        ) -> Result<(), ContractError> {
            self.env()
                .extension()
                .do_balance_transfer(amount, recipient)?;
            self.stored_number = 100;
            Ok(())
        }

        #[ink(message)]
        /// Get the free balance for the given account. Included mainly for testing
        pub fn get_balance(&mut self, account: AccountId) -> Result<u32, ContractError> {
            let value = self.env().extension().do_get_balance(account);
            self.env().emit_event(ResultNum { number: value? });
            self.stored_number = 75;
            ink_env::debug_println!("{:?}", &value);
            value
        }

        #[ink(message)]
        /// Get the current storage value. Included mainly for testing
        pub fn get_runtime_storage_value(&mut self) -> Result<u32, ContractError> {
            let value = self.env().extension().do_get_from_runtime();
            self.env().emit_event(ResultNum { number: value? });
            self.stored_number = 50;
            value
        }

        #[ink(message)]
        pub fn transfer_native(
            &mut self,
            destination: AccountId,
            value: Balance,
        ) -> Result<(), ContractError> {
            ink_env::transfer::<CustomEnvironment>(destination, value);
            Ok(())
        }

        /* /// Transfer value from the contract to the destination account ID
        #[ink(message)]
        pub fn deposit(&mut self, value: Balance) -> Result<(), ContractError> {
            let from: AccountId = self.env().account_id();
            let to: AccountId = self.caller_account();
            self.transfer_from_to(&from, &to, value);
            Ok(())
        }
        */

        //#[inline]
        #[ink(message)]
        pub fn give_me(&mut self, value: Balance) -> Result<(), ContractError> {
            self.env()
                .extension()
                .claim(self.env().account_id(), self.env().caller(), value);
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn chain_extension_store_in_runtime_works() {
            // given
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    1
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink_env::chain_extension::FromStatusCode` implementation for
                /// `RandomReadErr`.
                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let mut runtime_interface = RuntimeInterface::default();
            assert_eq!(runtime_interface.get(), 0);

            //when
            runtime_interface
                .store_in_runtime(32)
                .expect("update must work");

            // // then
            assert_eq!(runtime_interface.get(), 32);
        }

        #[ink::test]
        fn chain_extension_extended_transfer_works() {
            // given
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    2
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink_env::chain_extension::FromStatusCode` implementation for
                /// `RandomReadErr`.
                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let mut runtime_interface = RuntimeInterface::default();
            assert_eq!(runtime_interface.get(), 0);

            //when
            runtime_interface
                .extended_transfer(100, AccountId::from([0x1; 32]))
                .expect("update must work");

            // // then
            assert_eq!(runtime_interface.get(), 100);
        }

        #[ink::test]
        fn chain_extension_get_balance_works() {
            // given
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    3
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink_env::chain_extension::FromStatusCode` implementation for
                /// `RandomReadErr`.
                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let mut runtime_interface = RuntimeInterface::default();
            assert_eq!(runtime_interface.get(), 0);

            //when
            runtime_interface
                .get_balance(AccountId::from([0x1; 32]))
                .expect("update must work");

            // // then
            assert_eq!(runtime_interface.get(), 75);
        }

        #[ink::test]
        fn chain_extension_get_runtime_storage_value_works() {
            // given
            struct MockedExtension;
            impl ink_env::test::ChainExtension for MockedExtension {
                /// The static function id of the chain extension.
                fn func_id(&self) -> u32 {
                    4
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink_env::chain_extension::FromStatusCode` implementation for
                /// `RandomReadErr`.
                fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink_env::test::register_chain_extension(MockedExtension);
            let mut runtime_interface = RuntimeInterface::default();
            assert_eq!(runtime_interface.get(), 0);

            //when
            runtime_interface
                .get_runtime_storage_value()
                .expect("update must work");

            // // then
            assert_eq!(runtime_interface.get(), 50);
        }
    }
}
