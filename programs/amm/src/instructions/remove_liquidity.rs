use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, BurnChecked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::errors::AMMError;
use crate::state::Pool;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub liquidity_revoker: Signer<'info>,

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
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_revoker,
        associated_token::token_program = token_program,
    )]
    pub revoker_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_revoker,
        associated_token::token_program = token_program,
    )]
    pub revoker_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = liquidity_revoker,
        associated_token::token_program = token_program,
    )]
    pub revoker_token_lp: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
    minimum_token_a: u64,
    minimum_token_b: u64,
) -> Result<()> {
    // Security Check
    if ctx.accounts.revoker_token_lp.amount == 0 {
        return Err(AMMError::ZeroContriInPool.into());
    }
    // Store the required states
    let vault_a_amount = ctx.accounts.vault_a.amount;
    let vault_b_amount = ctx.accounts.vault_b.amount;
    let total_lp_supply = ctx.accounts.lp_mint.supply;
    let token_program = &mut ctx.accounts.token_program;

    // Calculate the amount of token_a and token_b w.r.t. lp_tokens revoker holds
    // required_tokens = lp_tokens alloted / total lp_tokens supply  * total amount in vault of that token

    let required_token_a = (lp_amount as u128)
        .checked_mul(vault_a_amount as u128)
        .unwrap()
        .checked_div(total_lp_supply as u128)
        .unwrap() as u64;
    let required_token_b = (lp_amount as u128)
        .checked_mul(vault_b_amount as u128)
        .unwrap()
        .checked_div(total_lp_supply as u128)
        .unwrap() as u64;

    // Check for Slippage
    require!(
        required_token_a >= minimum_token_a && required_token_b >= minimum_token_b,
        AMMError::SlippageExceededForLR
    );

    // Burn the liquidity providers Lp_tokens
    let burn_accounts = BurnChecked {
        mint: ctx.accounts.lp_mint.to_account_info(),
        from: ctx.accounts.revoker_token_lp.to_account_info(),
        authority: ctx.accounts.liquidity_revoker.to_account_info(),
    };
    let cpi_ctx_burn = CpiContext::new(token_program.to_account_info(), burn_accounts);
    let decimals = ctx.accounts.lp_mint.decimals;
    token_interface::burn_checked(cpi_ctx_burn, lp_amount, decimals)?;

    // Transfer the tokens from the vault to the liquidity revoker
    let mint_a_keys = ctx.accounts.token_a_mint.key();
    let mint_b_keys = ctx.accounts.token_b_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"pool",
        mint_a_keys.as_ref(),
        mint_b_keys.as_ref(),
        &[ctx.accounts.liquidity_pool.bump],
    ]];

    let decimals_a = ctx.accounts.token_a_mint.decimals;
    let decimals_b = ctx.accounts.token_b_mint.decimals;

    let transfer_a_accounts = TransferChecked {
        from: ctx.accounts.vault_a.to_account_info(),
        to: ctx.accounts.revoker_token_a.to_account_info(),
        mint: ctx.accounts.token_a_mint.to_account_info(),
        authority: ctx.accounts.liquidity_pool.to_account_info(),
    };
    let cpi_ctx_a = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer_a_accounts,
        signer_seeds,
    );

    token_interface::transfer_checked(cpi_ctx_a, required_token_a, decimals_a)?;

    let transfer_b_accounts = TransferChecked {
        from: ctx.accounts.vault_b.to_account_info(),
        to: ctx.accounts.revoker_token_b.to_account_info(),
        mint: ctx.accounts.token_b_mint.to_account_info(),
        authority: ctx.accounts.liquidity_pool.to_account_info(),
    };
    let cpi_ctx_b = CpiContext::new_with_signer(
        token_program.to_account_info(),
        transfer_b_accounts,
        signer_seeds,
    );

    token_interface::transfer_checked(cpi_ctx_b, required_token_b, decimals_b)?;

    Ok(())
}
