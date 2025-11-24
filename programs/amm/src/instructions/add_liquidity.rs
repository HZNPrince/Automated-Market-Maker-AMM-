use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::Pool;
use crate::errors::AMMError;
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    pub token_a_mint: InterfaceAccount<'info, Mint>,

    pub token_b_mint: InterfaceAccount<'info,Mint>,

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
    pub vault_a : InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_b", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = liquidity_provider,
        associated_token::token_program = token_program,
        
    )]
    pub provider_token_a : InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = liquidity_provider,
        associated_token::token_program = token_program,
        
    )]
    pub provider_token_b : InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub lp_mint : InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = liquidity_provider,
        associated_token::mint = lp_mint,
        associated_token::authority = liquidity_provider,
        associated_token::token_program = token_program,
    )]
    pub provider_token_lp: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


pub fn process_add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    minimum_lp_tokens: u64,
) -> Result<()> {

    // Firstly , Lets store the state 
    let vault_a_amount = ctx.accounts.vault_a.amount;
    let vault_b_amount = ctx.accounts.vault_b.amount;
    let total_lp_supply = ctx.accounts.lp_mint.supply;

    // Calculation of the lp needed to mint
    // required lp_tokens = amount_provided_for_pool / total amount in liquidity_pool's vault * total_supply of lp_tokens
    let required_lp_token_a = (amount_a as u128).checked_mul(total_lp_supply as u128).unwrap().checked_div(vault_a_amount as u128).unwrap();
    let required_lp_token_b = (amount_b as u128).checked_mul(total_lp_supply as u128).unwrap().checked_div(vault_b_amount as u128).unwrap();

    let lp_token_to_mint = std::cmp::min(required_lp_token_a, required_lp_token_b);

    require!(lp_token_to_mint >= minimum_lp_tokens as u128, AMMError::SlippageExceededForLP);

    // Calculate the amount required for transfer for the lp_token calculated
    let amount_to_transfer_a = (lp_token_to_mint).checked_mul(vault_a_amount as u128).unwrap().checked_div(total_lp_supply as u128).unwrap() as u64;
    let amount_to_transfer_b = (lp_token_to_mint).checked_mul(vault_b_amount as u128).unwrap().checked_div(total_lp_supply as u128).unwrap() as u64;

    // Security check for user providing enough amount of both tokens
    require!(amount_a >= amount_to_transfer_a && amount_b >= amount_to_transfer_b , AMMError::InsufficientAmount);

    // Transfer the amount to the specific liquidity-pool vaults 
    let transfer_a_accounts = TransferChecked{
        from: ctx.accounts.provider_token_a.to_account_info(),
        to: ctx.accounts.vault_a.to_account_info(),
        mint: ctx.accounts.token_a_mint.to_account_info(),
        authority: ctx.accounts.liquidity_provider.to_account_info(),
    };
    let cpi_ctx_a = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_a_accounts);
    let decimals_a = ctx.accounts.token_a_mint.decimals;
    token_interface::transfer_checked(cpi_ctx_a, amount_to_transfer_a, decimals_a)?;

    let transfer_b_accounts = TransferChecked{
        from: ctx.accounts.provider_token_b.to_account_info(),
        to: ctx.accounts.vault_b.to_account_info(),
        mint: ctx.accounts.token_b_mint.to_account_info(),
        authority: ctx.accounts.liquidity_provider.to_account_info(),
    };

    let cpi_ctx_b = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_b_accounts);
    let decimals_b = ctx.accounts.token_b_mint.decimals;
    token_interface::transfer_checked(cpi_ctx_b, amount_to_transfer_b, decimals_b)?;

    // Mint the alloted lp_tokens to the liquidity provider 
    let mint_to_accounts = MintTo{
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.provider_token_lp.to_account_info(),
        authority: ctx.accounts.liquidity_pool.to_account_info(),
    };
    let mint_a_keys = ctx.accounts.token_a_mint.key();
    let mint_b_keys = ctx.accounts.token_b_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a_keys.as_ref(), mint_b_keys.as_ref(), &[ctx.accounts.liquidity_pool.bump]]];
    let cpi_ctx_lp = CpiContext::new(ctx.accounts.token_program.to_account_info(), mint_to_accounts).with_signer(signer_seeds);

    token_interface::mint_to(cpi_ctx_lp, lp_token_to_mint as u64)?;


    Ok(())
}
