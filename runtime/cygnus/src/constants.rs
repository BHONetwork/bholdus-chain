//! A set of constant values used in substrate runtime.

/// Money matters.
pub mod currency {
    use bholdus_primitives::Balance;

    pub const TOKEN_DECIMALS: u32 = 18;
    pub const TOKEN_SYMBOL: &str = "BHOT";
    pub const UNITS: Balance = 10_u128.pow(TOKEN_DECIMALS);
    pub const BHO: Balance = UNITS;
    pub const DOLLARS: Balance = UNITS;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Time.
pub mod time {
    use bholdus_primitives::{BlockNumber, Moment};

    /// Since BABE is probabilistic this is the average expected block time that
    /// we are targeting. Blocks will be produced at a minimum duration defined
    /// by `SLOT_DURATION`, but some slots will not be allocated to any
    /// authority and hence no block will be produced. We expect to have this
    /// block time on average following the defined slot duration and the value
    /// of `c` configured for BABE (where `1 - c` represents the probability of
    /// a slot being empty).
    /// This value is only used indirectly to define the unit constants below
    /// that are expressed in blocks. The rest of the code should use
    /// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
    /// minimum period).
    ///
    /// If using BABE with secondary slots (default) then all of the slots will
    /// always be assigned, in which case `MILLISECS_PER_BLOCK` and
    /// `SLOT_DURATION` should have the same value.
    ///
    /// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: Moment = 3000;
    pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

    // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}
