use crate::*;

pub const GLOBAL_SEED: &[u8] = b"global";
pub const HOURLY_SLOTS: u16 = 9000; // ~1 hour at 400 ms/slot
pub const MAX_RAND_VALUE: u8 = 254;

#[account]
pub struct Global {
    // GLOBAL SETTINGS
    pub hour_to_next_update: u8, // ===============> MIN 1HR - MAX 85 HRS | Result / 3
    /// The slot when the current guess was placed.
    pub next_update_slot: u64,
    /// Token Mint.
    pub mint: Pubkey,

    // SWITCHBOARD SETTINGS
    // Switchboard Function Request pubkey.
    pub switchboard_function: Pubkey,
    // Switchboard Function Request pubkey.
    pub attestation_program_state: Pubkey,
    // Switchboard Function Request pubkey.
    pub attestation_queue: Pubkey,
    // Switchboard Function Request pubkey.
    pub switchboard_request: Option<Pubkey>,

    // TRANSFER FEE SETTINGS
    /// The max transfer fee in basis point.
    pub current_transfer_fee_bp: u16, // ===============> 0 - 60%, 10% Interval
}
