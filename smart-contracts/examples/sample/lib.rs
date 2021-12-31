#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::{AccountId, Environment};
use ink_lang as ink;

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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    FailToCallRuntime,
    UnknownStatusCode,
    InvalidScaleEncoding,
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
    use super::ContractError;

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
        pub fn get(&self) -> u32 {
            self.stored_number
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
