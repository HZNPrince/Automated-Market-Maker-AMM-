use crate::errors::AMMError;
use crate::state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub token_a_mint: InterfaceAccount<'info, Mint>,

    pub token_b_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"vault_a", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_b", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub swap_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = token_b_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub swap_account_b: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_swap(
    ctx: Context<Swap>,
    amount_in: u64,
    minimum_amount_out: u64,
    a_to_b: bool,
) -> Result<()> {
    // Set the correct variables as per the direction of transfer
    let (vault_a, vault_b, user_token_a, user_token_b, mint_a, mint_b) = match a_to_b {
        true => (
            &ctx.accounts.vault_a,
            &ctx.accounts.vault_b,
            &ctx.accounts.swap_account_a,
            &ctx.accounts.swap_account_b,
            &ctx.accounts.token_a_mint,
            &ctx.accounts.token_b_mint,
        ),
        false => (
            &ctx.accounts.vault_b,
            &ctx.accounts.vault_a,
            &ctx.accounts.swap_account_b,
            &ctx.accounts.swap_account_a,
            &ctx.accounts.token_b_mint,
            &ctx.accounts.token_a_mint,
        ),
    };

    // Store the vault amounts
    let reserve_in = vault_a.amount;
    let reserve_out = vault_b.amount;

    // Transfer swap amount to the vault_a
    let transfer_vault_accounts = TransferChecked {
        from: user_token_a.to_account_info(),
        to: vault_a.to_account_info(),
        mint: mint_a.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_ctx_vault_tranfer = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_vault_accounts,
    );

    let decimals = mint_a.decimals;
    token_interface::transfer_checked(cpi_ctx_vault_tranfer, amount_in, decimals)?;

    // Since the user swap amount is in the correct vault ,
    // we can now proceed with the calculation of the transfer

    let amount_out = calculate_output_amount(reserve_in, reserve_out, amount_in)?;

    require!(amount_out >= minimum_amount_out, AMMError::SlippageExceeded);

    // The calculated amount will now be transfer to the swapper from vault_b
    let transfer_swapped_accounts = TransferChecked {
        from: vault_b.to_account_info(),
        to: user_token_b.to_account_info(),
        mint: mint_b.to_account_info(),
        authority: ctx.accounts.liquidity_pool.to_account_info(),
    };
    let mint_a_keys = ctx.accounts.token_a_mint.key();
    let mint_b_keys = ctx.accounts.token_b_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool",
        mint_a_keys.as_ref(),
        mint_b_keys.as_ref(),
        &[ctx.accounts.liquidity_pool.bump],
    ]];
    let cpi_ctx_swapped = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_swapped_accounts,
    )
    .with_signer(signer_seeds);

    let decimals = mint_b.decimals;
    token_interface::transfer_checked(cpi_ctx_swapped, amount_out, decimals)?;

    Ok(())
}

fn calculate_output_amount(vault_a: u64, vault_b: u64, swap_amount: u64) -> Result<u64> {
    require!(vault_a > 0 && vault_b > 0, AMMError::InsufficientLiquidity);
    require!(swap_amount > 0, AMMError::InvalidInput);

    // Taking 0.3% fee for liquidity providers
    let swap_amount_with_fee = (swap_amount as u128)
        .checked_mul(997)
        .unwrap()
        .checked_div(1000)
        .unwrap();

    // k = x * y
    let k = (vault_a as u128).checked_mul(vault_b as u128).unwrap();
    let new_vault_a_amount = (vault_a as u128).checked_add(swap_amount_with_fee).unwrap();
    let new_vault_b_amount = k.checked_div(new_vault_a_amount).unwrap();
    let transfer_amount = (vault_b as u128).checked_sub(new_vault_b_amount).unwrap();

    Ok(transfer_amount as u64)
}
