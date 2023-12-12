use crate::{Global, RandomnessRequestError, GLOBAL_SEED};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::Token2022};
use switchboard_solana::{
    AttestationProgramState, AttestationQueueAccountData, FunctionAccountData,
    FunctionRequestAccountData, Mint, Token, TokenAccount, SWITCHBOARD_ATTESTATION_PROGRAM_ID,
};

// 1. InitManagerState
// 2. UpdateMintManager
// 3. UpdateAuthority
// 3. RemoveAuthority
// 4. TriggerUpdate
// 5. CallbackFunction

// 1. ONLY CALLED ONCE
#[derive(Accounts)]
pub struct InitGlobal<'info> {
    // RANDOMNESS PROGRAM ACCOUNTS
    #[account(
    init_if_needed,
    seeds = [GLOBAL_SEED],
    bump,
    payer = payer,
    space = 8 + std::mem::size_of::<Global>(),
  )]
    pub global: Box<Account<'info, Global>>,
    /// CHECK:
    // #[account(mint::token_program = token_2022::ID)]
    pub mint: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    // SWITCHBOARD ACCOUNTS
    /// CHECK:
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard: AccountInfo<'info>,
    /// CHECK: validated by Switchboard CPI
    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,
    /// CHECK: validated by Switchboard CPI
    #[account(mut)]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_request.data_len() == 0 && switchboard_request.lamports() == 0
      )]
    pub switchboard_request: AccountInfo<'info>,
    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_request_escrow.data_len() == 0 && switchboard_request_escrow.lamports() == 0
      )]
    pub switchboard_request_escrow: AccountInfo<'info>,

    // TOKEN ACCOUNTS
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub switchboard_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub token_program_22: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // SYSTEM ACCOUNTS
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CollectAndBurn<'info> {
    #[account(mut, seeds = [GLOBAL_SEED], bump)]
    pub global: Account<'info, Global>,
    #[account(mut, associated_token::authority = global, associated_token::mint = mint)]
    pub global_ata: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct TriggerUpdate<'info> {
    // RANDOMNESS PROGRAM ACCOUNTS
    #[account(
      mut, seeds = [GLOBAL_SEED], bump,
      constraint = global.next_update_slot >= Clock::get()?.slot as u64 @ RandomnessRequestError::RequestNotReady
    )]
    pub global: Box<Account<'info, Global>>,
    /// CHECK:
    pub mint: AccountInfo<'info>,

    pub enclave_signer: Signer<'info>,

    // SWITCHBOARD ACCOUNTS
    /// CHECK:
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard: AccountInfo<'info>,
    /// CHECK: validated by Switchboard CPI
    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,
    /// CHECK: validated by Switchboard CPI
    #[account(mut)]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    /// CHECK: validated by Switchboard CPI
    #[account(
      constraint = switchboard_request.validate_signer(
        &switchboard_function,
        &enclave_signer.to_account_info()
        )?
      )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_request_escrow.data_len() == 0 && switchboard_request_escrow.lamports() == 0
      )]
    pub switchboard_request_escrow: AccountInfo<'info>,

    // TOKEN ACCOUNTS
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub switchboard_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub token_program_22: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // SYSTEM ACCOUNTS
    pub system_program: Program<'info, System>,
}
