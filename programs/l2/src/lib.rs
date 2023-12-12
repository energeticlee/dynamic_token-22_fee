use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn};
use spl_token_2022::extension::transfer_fee::instruction::{
    set_transfer_fee, withdraw_withheld_tokens_from_accounts, withdraw_withheld_tokens_from_mint,
};
use switchboard_solana::{invoke_signed, FunctionRequestInit, FunctionRequestTrigger};

pub mod error;
pub use error::*;

pub mod context;
pub use context::*;

pub mod state;
pub use state::*;

pub mod utils;
pub use utils::*;

declare_id!("auULn3TunUFz5mvM1VSLUT184oAApgnEsLmqZrVyUAP");
// WITHHELD_WITHDRAW & TRANSFER_FEE AUTHORITY REQUIRED TO BE GLOBAL PDA
// MINT MANAGER AUTHORITY CONSTRAINT CHECK TO UPDATE

#[program]
pub mod l2 {

    use super::*;

    // INITIALIZE GLOBAL, MINT REQUIRE TRANSFER_FEE & MINT AUTHORITY
    pub fn init_global(ctx: Context<InitGlobal>, hour_to_next_update: u8) -> Result<()> {
        // TODO: CHECK MINT
        let global = &mut ctx.accounts.global;
        let request_params = format!(
            "PID={},MAX_VALUE={},GLOBAL={},MINT={},",
            crate::id(),
            MAX_RAND_VALUE,
            global.key(),
            ctx.accounts.mint.key()
        );
        let container_params = request_params.into_bytes();

        // Create the Switchboard request account.
        let request_init_ctx = FunctionRequestInit {
            request: ctx.accounts.switchboard_request.clone(),
            authority: global.to_account_info(), // AUTHORITY OVER REQUEST
            function: ctx.accounts.switchboard_function.to_account_info(),
            function_authority: None,
            escrow: ctx.accounts.switchboard_request_escrow.clone(), // ESCROW ACCOUNT THAT WILL BE PAYING FOR FEES
            mint: ctx.accounts.switchboard_mint.to_account_info(),   // NATIVE SOL MINT
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };

        request_init_ctx.invoke(
            ctx.accounts.switchboard.clone(),
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(container_params.len() as u32),
            // container_params - the container params
            // default: empty vec
            Some(container_params),
            // garbage_collection_slot - the slot when the request can be closed by anyone and is considered dead
            // default: None, only authority can close the request
            None,
        )?;

        let current_slot = Clock::get()?.slot as u64;
        // let new_next_update = current_slot + HOURLY_SLOTS as u64 * hour_to_next_update as u64;
        let new_next_update = current_slot + 5; // 10 Seconds

        // Then trigger it
        // We do this in two steps so we can set the authority to our Lottery PDA
        let trigger_ctx = FunctionRequestTrigger {
            request: ctx.accounts.switchboard_request.to_account_info(),
            authority: global.to_account_info(),
            escrow: ctx.accounts.switchboard_request_escrow.to_account_info(),
            function: ctx.accounts.switchboard_function.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        trigger_ctx.invoke_signed(
            ctx.accounts.switchboard.clone(),
            // bounty - the amount of SOL to pay the Switchboard Function for executing the request
            None,
            // slots_until_expiration - the number of slots until the request expires
            None,
            // valid_after_slot - the slot when the request can be executed
            Some(new_next_update),
            // Lottery PDA seeds
            &[&[GLOBAL_SEED, &[ctx.bumps.global]]],
        )?;

        global.hour_to_next_update = hour_to_next_update;
        global.next_update_slot = new_next_update;
        global.mint = ctx.accounts.mint.key();

        global.switchboard_function = ctx.accounts.switchboard_function.key();
        global.attestation_program_state = ctx.accounts.switchboard_state.key();
        global.attestation_queue = ctx.accounts.switchboard_attestation_queue.key();
        global.switchboard_request = None;

        global.current_transfer_fee_bp = 100_00; // 100%

        Ok(())
    }

    // TODO: WITHDRAW WITHHELD TOKENS => Anyone can trigger
    // MINT GOVERNANCE => Send withdraw_withheld_tokens to incinerator
    pub fn collect_and_burn_from_account(ctx: Context<CollectAndBurn>) -> Result<()> {
        let accounts: Vec<&Pubkey> = ctx.remaining_accounts.iter().map(|acc| acc.key).collect();

        let seeds = &[GLOBAL_SEED, &[ctx.bumps.global]];
        let withdraw_ix = withdraw_withheld_tokens_from_accounts(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.global_ata.key(),
            &ctx.accounts.global.key(),
            &[&ctx.accounts.global.key()],
            &accounts,
        )?;
        invoke_signed(
            &withdraw_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.global.to_account_info(),
                ctx.accounts.global_ata.to_account_info(),
            ],
            &[seeds],
        )?;

        // BURN
        let cpi_program = ctx.accounts.token_program.to_account_info(); // The program that we are calling
        let cpi_accounts = Burn {
            // The instruction followed by the parameters required
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.global_ata.to_account_info(),
            authority: ctx.accounts.global.to_account_info(),
        };
        // ::new since the signer has already sign the transaction
        burn(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, &[seeds]),
            ctx.accounts.global_ata.amount,
        )?;

        Ok(())
    }
    pub fn collect_and_burn_from_mint(ctx: Context<CollectAndBurn>) -> Result<()> {
        let seeds = &[GLOBAL_SEED, &[ctx.bumps.global]];
        let withdraw_ix = withdraw_withheld_tokens_from_mint(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.global_ata.key(),
            &ctx.accounts.global.key(),
            &[&ctx.accounts.global.key()],
        )?;
        invoke_signed(
            &withdraw_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.global.to_account_info(),
                ctx.accounts.global_ata.to_account_info(),
            ],
            &[seeds],
        )?;

        // BURN
        let cpi_program = ctx.accounts.token_program.to_account_info(); // The program that we are calling
        let cpi_accounts = Burn {
            // The instruction followed by the parameters required
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.global_ata.to_account_info(),
            authority: ctx.accounts.global.to_account_info(),
        };
        // ::new since the signer has already sign the transaction
        burn(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, &[seeds]),
            ctx.accounts.global_ata.amount,
        )?;

        Ok(())
    }

    pub fn trigger_update(ctx: Context<TriggerUpdate>, result: u64) -> anchor_lang::Result<()> {
        let global = &mut ctx.accounts.global;
        let result8 = result as u8;

        msg!("CRANK TRIGGER");
        if !(0..MAX_RAND_VALUE).contains(&result8) {
            return Err(error!(RandomnessRequestError::RandomResultOutOfBounds));
        }

        let current_slot = Clock::get()?.slot as u64;

        // Update next_update_slot & update hour_to_next_update
        let hour_to_next_update = (((current_slot + result8 as u64) % 24) + 1) as u8;
        let new_next_update = current_slot + (HOURLY_SLOTS * hour_to_next_update as u16) as u64;
        global.hour_to_next_update = hour_to_next_update;
        global.next_update_slot = new_next_update;

        // Update mint with new current_transfer_fee_bp
        let new_transfer_fee = result as u16 % 7 * 10 * 100;
        global.current_transfer_fee_bp = new_transfer_fee;
        // UPDATE MINT TRANSFER FEE
        let seeds = &[GLOBAL_SEED, &[ctx.bumps.global]];
        let update_transfer_fee_ix = set_transfer_fee(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &global.key(),
            &[&global.key()],
            new_transfer_fee,
            0,
        )?;
        invoke_signed(
            &update_transfer_fee_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                global.to_account_info(),
            ],
            &[seeds],
        )?;

        // Trigger the Switchboard request
        // This will instruct the off-chain oracles to execute your docker container and relay
        // the result back to our program via the 'settle' instruction.
        let trigger_ctx = FunctionRequestTrigger {
            request: ctx.accounts.switchboard_request.to_account_info(),
            authority: global.to_account_info(),
            escrow: ctx.accounts.switchboard_request_escrow.to_account_info(),
            function: ctx.accounts.switchboard_function.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: global.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        trigger_ctx.invoke_signed(
            ctx.accounts.switchboard.clone(),
            // bounty - the amount of SOL to pay the Switchboard Function for executing the request
            None,
            // slots_until_expiration - the number of slots until the request expires
            None,
            // valid_after_slot - the slot when the request can be executed
            Some(new_next_update),
            // Lottery PDA seeds
            &[&[GLOBAL_SEED, &[ctx.bumps.global]]],
        )?;

        Ok(())
    }
}
