use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

pub fn transfer<'a>(
    token_program: &AccountInfo<'a>,
    from: &Account<'a, TokenAccount>,
    to: &Account<'a, TokenAccount>,
    authority: &AccountInfo<'a>,
    auth_seed: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let cpi_program = token_program.clone();
    let cpi_accounts = anchor_spl::token::Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, auth_seed);
    anchor_spl::token::transfer(cpi_ctx, amount)?;
    Ok(())
}

pub fn calculate_reward(annual_return_bps: u16, time_period_hours: u8, amount: u64) -> u64 {
    // Convert annual_return_bps and time_period_hours to decimal values
    let annual_return_decimal = f64::from(annual_return_bps) / 10000.0;
    let time_period_in_years = f64::from(time_period_hours) / 8760.0;

    // Calculate the amount rewarded using the compound interest formula
    let reward = (1.0 + annual_return_decimal).powf(time_period_in_years) * amount as f64;

    // Convert the result back to u64
    reward as u64
}
