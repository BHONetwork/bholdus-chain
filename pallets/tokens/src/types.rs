//! Various basic types for use in the assets pallet.

use super::*;
use frame_support::pallet_prelude::*;

pub(super) type DepositBalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T, I = ()> = <<T as Config<I>>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub struct AssetDetails<Balance, AccountId, DepositBalance> {
    /// Can change `owner`, issuer, `freezer` and `admin` accounts.
    pub(super) owner: AccountId,
    /// Can mint tokens.
    pub(super) issuer: AccountId,
    /// Can thaw tokens, force transfers and burn tokens from any account.
    pub(super) admin: AccountId,
    /// Can freeze tokens.
    pub(super) freezer: AccountId,
    /// The total supply across all accounts.
    pub(super) supply: Balance,
    /// The balance deposited for this asset. This pays for the data stored here.
    pub(super) deposit: DepositBalance,
    /// The ED for virtual accounts.
    pub(super) min_balance: Balance,
    /// If `true`, then any account with this asset is given a provider reference. Otherwise, it
    /// requires a consumer reference.
    pub(super) is_sufficient: bool,
    /// The total number of accounts.
    pub(super) accounts: u32,
    /// The total number of accounts for which we have placed a self-sufficient reference.
    pub(super) sufficients: u32,
    /// The total number of approvals.
    pub(super) approvals: u32,
    /// Whether the asset is frozen for non-admin transfers.
    pub(super) is_frozen: bool,
}

impl<Balance, AccountId, DepositBalance> AssetDetails<Balance, AccountId, DepositBalance> {
    pub fn destroy_witness(&self) -> DestroyWitness {
        DestroyWitness {
            accounts: self.accounts,
            sufficients: self.sufficients,
            approvals: self.approvals,
        }
    }
}

/// Data concering an approval.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen)]
pub struct Approval<Balance, DepositBalance> {
    /// The amount ow funds approved for the balance transfer from the owner to some delegated
    /// target.
    pub(super) amount: Balance,
    /// The amount reserved on the owner's account to hold this item in storage.
    pub(super) deposit: DepositBalance,
}

/// Trait for allowing a minimum balance on the account to be specified, beyond the
/// `minimum_balance` of the asset. This is additive - the `minimum_balance` of the asset must be
/// met *and then* anything here in addition.
pub trait FrozenBalance<AssetId, AccountId, Balance> {
    /// Return the fronzen balance. Under normal behaviour, this amount should always be
    /// withdrawable.
    ///
    /// In reality, the balance of every account must be at least the sum of this (if `Some`) and
    /// the asset's minimum_balance, since there may be complications to destroying an asset's
    /// account completely.
    ///
    /// If `None` is returned, then nothing special is enforced.
    ///
    /// If any operation ever breaks this requirement (which will only happen through some sort of
    /// privileged intervention), then `melted` is called to do any cleanup.
    fn frozen_balance(asset: AssetId, who: &AccountId) -> Option<Balance>;
    /// Called when an account has been removed.
    fn died(asset: AssetId, who: &AccountId);
}

impl<AssetId, AccountId, Balance> FrozenBalance<AssetId, AccountId, Balance> for () {
    fn frozen_balance(_: AssetId, _: &AccountId) -> Option<Balance> {
        None
    }
    fn died(_: AssetId, _: &AccountId) {}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct TransferFlags {
    /// The debited account must stay alive at the end of the operation; an error is returned if
    /// this cannot be achieved legally.
    pub(super) keep_alive: bool,
    /// Less than the amount specified needs be debited by the operation for it to be considered
    /// successful. If `false`, then the amount debited will always be at least the amount
    /// specified.
    pub(super) best_effort: bool,
    /// Any additional funds debited (due to minimum balance requirements) should be burned rather
    /// than credited to the destination account.
    pub(super) burn_dust: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct DebitFlags {
    /// The debited account must stay alive at the end of the operation; an error is returned if
    /// this cannot be achieved legally.
    pub(super) keep_alive: bool,
    /// Less than the amount specified needs be debited by the operation for it to be considered
    /// successful. If `false`, then the amount debited will always be at least the amount
    /// specified.
    pub(super) best_effort: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AssetBalance<Balance, Extra> {
    /// The balance.
    pub(super) balance: Balance,
    /// Whether the account is frozen.
    pub(super) is_frozen: bool,
    /// `true` if this balance gave the account a self-sufficient reference.
    pub(super) sufficient: bool,
    /// Additional "sidecar" data, in case some other pallet wants to use this storage item.
    pub(super) extra: Extra,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AssetMetadata<DepositBalance, BoundedString> {
    /// The balance deposited for this metadata.
    ///
    /// This pays for the data stored in this struct.
    pub(super) deposit: DepositBalance,
    /// The user friendly name of this asset. Limited in length by `StringLimit`.
    pub(super) name: BoundedString,
    /// The ticker symbol for this asset. Limited in length by `StringLimit`.
    pub(super) symbol: BoundedString,
    /// The number of decimals this asset uses to represent on unit.
    pub(super) decimals: u8,
    /// Whether the asset metadata may be changed by a non Force origin.
    pub(super) is_frozen: bool,
}

/// Witness data for the destroy transactions.
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct DestroyWitness {
    /// The number of accounts holding the asset.
    #[codec(compact)]
    pub(super) accounts: u32,
    /// The number of accounts holding the asset with a self-sufficient reference.
    #[codec(compact)]
    pub(super) sufficients: u32,
    /// The number of transfer-approvals of the asset.
    #[codec(compact)]
    pub(super) approvals: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct AssetProfile<DepositBalance, BoundedString> {
    /// The balance deposited for this metadata.
    ///
    /// This pays for the data stored in this struct.
    pub(super) deposit: DepositBalance,
    /// The user friendly name of this asset. Limited in length by `StringLimit`.
    pub(super) name: BoundedString,
    /// The ticker symbol for this asset. Limited in length by `StringLimit`.
    /// Whether the asset metadata may be changed by a non Force origin.
    pub(super) is_frozen: bool,
}

impl From<TransferFlags> for DebitFlags {
    fn from(f: TransferFlags) -> Self {
        Self {
            keep_alive: f.keep_alive,
            best_effort: f.best_effort,
        }
    }
}

// Either underlying data blob if it is at most 32 bytes, or a hash of it. If the data is greater
// than 32-bytes then it will be truncated when encoding.
//
// Can also be `None`.

#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Data {
    /// No data here.
    None,
    /// The data is stored directly.
    Raw(Vec<u8>),
    /// Only the Blake2 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    BlakeTwo256([u8; 32]),
    /// Only the SHA2-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Sha256([u8; 32]),
    /// Only the Keccak-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    Keccak256([u8; 32]),
    /// Only the SHA3-256 hash of the data is stored. The preimage of the hash may be retrieved
    /// through some hash-lookup service.
    ShaThree256([u8; 32]),
}

impl Decode for Data {
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let b = input.read_byte()?;
        Ok(match b {
            0 => Data::None,
            n @ 1..=33 => {
                let mut r = vec![0u8; n as usize - 1];
                input.read(&mut r[..])?;
                Data::Raw(r)
            }
            34 => Data::BlakeTwo256(<[u8; 32]>::decode(input)?),
            35 => Data::Sha256(<[u8; 32]>::decode(input)?),
            36 => Data::Keccak256(<[u8; 32]>::decode(input)?),
            37 => Data::ShaThree256(<[u8; 32]>::decode(input)?),
            _ => return Err(codec::Error::from("invalid leading byte")),
        })
    }
}

impl Encode for Data {
    fn encode(&self) -> Vec<u8> {
        match self {
            Data::None => vec![0u8; 1],
            Data::Raw(ref x) => {
                let l = x.len().min(32);
                let mut r = vec![l as u8 + 1; l + 1];
                &mut r[1..].copy_from_slice(&x[..l as usize]);
                r
            }
            Data::BlakeTwo256(ref h) => once(34u8).chain(h.iter().cloned()).collect(),
            Data::Sha256(ref h) => once(35u8).chain(h.iter().cloned()).collect(),
            Data::Keccak256(ref h) => once(36u8).chain(h.iter().cloned()).collect(),
            Data::ShaThree256(ref h) => once(37u8).chain(h.iter().cloned()).collect(),
        }
    }
}

impl codec::EncodeLike for Data {}
impl Default for Data {
    fn default() -> Self {
        Self::None
    }
}

/// Information concerning the identity of the controller of an account.
///
/// NOTE: This is should be stored at the end of the storage item to facilitate the addition of
/// extra fields in a backwards compatible way through a specialized `Decode` impl.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(test, derive(Default))]
pub struct AssetIdentity {
    /// Additional fields of the identity that are not catered for with the struct's explicit
    /// fields.
    pub additional: Vec<(Data, Data)>,
    pub basic_information: BasicInformation,
    /// Social Profile
    pub social_profiles: SocialProfile,
    //    /// A reasonable display name for the controller of the account. This could be whatever it is
    //    /// that it is typically known as and should not be confusable with other entities, given
    //    /// reasonable context.
    //    ///
    //    /// Stored as UTF-8.
    //    pub project_name: Data,
    //    /// A representative website held by the controller of the account.
    //    ///
    //    /// NOTE: `https://` is automatically prepended.
    //    ///
    //    /// Stored as UTF-8.
    //    pub official_project_website: Data,
    //    /// The email address of the controller of the account.
    //    ///
    //    /// Stored as UTF-8.
    //    pub official_email_address: Data,
    //    /// Logo icon
    //    pub logo_icon: Data,
    //    /// Project Sector
    //    pub project_sector: Data,
    //    /// Project Description
    //    pub project_description: Data,
    //    /// Social Profile
    //    social_profile: SocialProfile,
}
/// NOTE: This should be stored at the end of the storage item to facilitate the addition of extra
/// fields in a backwards compatible way through a specialized `Decod` impl.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(test, derive(Default))]
pub struct BasicInformation {
    /// Stored as UTF-8.
    pub project_name: Data,
    /// A representative website held by the controller of the account.
    ///
    /// NOTE: `https://` is automatically prepended.
    ///
    /// Stored as UTF-8.
    pub official_project_website: Data,
    /// The email address of the controller of the account.
    ///
    /// Stored as UTF-8.
    pub official_email_address: Data,
    /// Logo icon
    pub logo_icon: Data,
    /// Project Sector
    pub project_sector: Data,
    /// Project Description
    pub project_description: Data,
}

/// NOTE: Social Profile
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(test, derive(Default))]
pub struct SocialProfile {
    /// Wnitepaper
    pub whitepaper: Data,
    /// Medium
    pub medium: Data,
    /// Github
    pub github: Data,
    /// Reddit
    pub reddit: Data,
    /// Telegram
    pub telegram: Data,
    /// Discord
    pub discord: Data,
    /// Slack
    pub slack: Data,
    /// Facebook
    /// NOTE: `https://` is automatically prepended.
    ///
    /// Stored as UTF-8.
    pub facebook: Data,
    /// Linkedin
    pub linkedin: Data,
    /// The Twitter identity. The leading `@` character may be elided.
    pub twitter: Data,
}

/// Information concerning the identity of the controller of an account.
///
/// NOTE: This is stored separately primarily to facilitate the addition of extra fields in a
/// backwards compatible way through a specialized `Decode` impl.
#[derive(Clone, Encode, Eq, PartialEq, RuntimeDebug)]
pub struct Registration<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> {
    /// Judgements from the registrars on this identity. Stored ordered by `RegistrarIndex`. There
    /// may be only a single judgement from each registrar.
    ///
    /// Amount held on deposit for this information.
    pub deposit: Balance,
    /// Information on the identity.
    pub info: AssetIdentity,
    pub is_verifiable: bool,
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq + Zero + Add>
    Registration<Balance>
{
    pub fn total_deposit(&self) -> Balance {
        self.deposit
    }
}

impl<Balance: Encode + Decode + Copy + Clone + Debug + Eq + PartialEq> Decode
    for Registration<Balance>
{
    fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
        let (deposit, info, is_verifiable) = Decode::decode(&mut AppendZerosInput::new(input))?;
        Ok(Self {
            deposit,
            info,
            is_verifiable,
        })
    }
}
