#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use ink_env::{AccountId, Environment};

#[ink::chain_extension]
pub trait ChainExtension {
    type ErrorCode = ContractError;
    
    #[ink(extension = 1)]
    fn transfer(
        tokenID: u64,
        amount: <ink_env::DefaultEnvironment as Environment>::Balance,
        recipient: AccountId
    ) -> Result<u32, ContractError>;
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
mod tokens {
    use super::ContractError;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Tokens {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl Tokens {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        // Invoke the extended transfer function with the arguments given to the smart contract
        // function
        #[ink(message)]
        pub fn extended_transfer(
            &mut self,
            tokenID: u64,
            amount: Balance,
            recipient: AccountId,
        ) -> Result<(), ContractError> {
            self.env()
                .extension()
                .transfer(tokenID, amount, recipient)?;
            self.value = true;
            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let tokens = Tokens::default();
            assert_eq!(tokens.get(), false);
        }

        #[ink::test]
        fn chain_extension_transfer_works() {
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
            let mut token = Tokens::default();
            assert_eq!(token.get(), false);

            //when
            token
                .extended_transfer(1, 100, AccountId::from([0x1; 32]))
                .expect("update must work");

            // // then
            assert_eq!(token.get(), True);
        }
        
    }
}
