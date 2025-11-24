use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::Pool;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub token_a_mint: InterfaceAccount<'info, Mint>,

    pub token_b_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        token::mint = token_a_mint,
        token::authority = liquidity_pool,
        token::token_program = token_program,
        seeds = [b"vault_a", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        token::mint = token_b_mint,
        token::authority = liquidity_pool,
        token::token_program = token_program,
        seeds = [b"vault_b", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub liquidity_pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub creator_token_a: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub creator_token_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 6,
        mint::authority = liquidity_pool,
        mint::token_program = token_program,
        seeds = [b"lp_mint", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
    )]
    pub lp_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program 
    )]
    pub creator_lp_token : InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program : Program<'info, AssociatedToken>,
}

pub fn process_initialize_pool(
    ctx: Context<InitializePool>,
    token_a_amount: u64,
    token_b_amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.liquidity_pool;
    let vault_a = &mut ctx.accounts.vault_a;
    let vault_b = &mut ctx.accounts.vault_b;
    let creator_token_a = &mut ctx.accounts.creator_token_a;
    let creator_token_b = &mut ctx.accounts.creator_token_b;
    let mint_a = &mut ctx.accounts.token_a_mint;
    let mint_b = &mut ctx.accounts.token_b_mint;
    let token_program = &mut ctx.accounts.token_program;

    // Change the state of the pool
    pool.token_a_mint = mint_a.key();
    pool.token_b_mint = mint_b.key();
    pool.vault_a = vault_a.key();
    pool.vault_b = vault_b.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();
    pool.bump = ctx.bumps.liquidity_pool;

    // Transfer tokens from creater to the vault
    // Token A : Creator to vault_A
    let transfer_token_a_accounts = TransferChecked {
        from: creator_token_a.to_account_info(),
        to: vault_a.to_account_info(),
        mint: mint_a.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };
    let cpi_ctx_token_a =
        CpiContext::new(token_program.to_account_info(), transfer_token_a_accounts);

    let decimals_token_a = mint_a.decimals;
    token_interface::transfer_checked(cpi_ctx_token_a, token_a_amount, decimals_token_a)?;

    // Token B : Creator to vault_B
    let transfer_token_b_accounts = TransferChecked {
        from: creator_token_b.to_account_info(),
        to: vault_b.to_account_info(),
        mint: mint_b.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };
    let cpi_ctx_token_b =
        CpiContext::new(token_program.to_account_info(), transfer_token_b_accounts);

    let decimals_token_b = mint_b.decimals;

    token_interface::transfer_checked(cpi_ctx_token_b, token_b_amount, decimals_token_b)?;

    // To Calculate and Mint LP tokens to the creator 
    // Calculation : sqrt(x * y)
    let lp_token_amount = f64::sqrt(token_a_amount as f64 * token_b_amount as f64) as u64;

    // Mint 
    let mint_to_accounts = MintTo{
        mint: ctx.accounts.lp_mint.to_account_info(),
        to: ctx.accounts.creator_lp_token.to_account_info(),
        authority: pool.to_account_info()
    };
    let mint_a_key = mint_a.key();
    let mint_b_key = mint_b.key();
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a_key.as_ref(), mint_b_key.as_ref(), &[ctx.bumps.liquidity_pool]]];
    let cpi_ctx_mint_to = CpiContext::new(token_program.to_account_info(), mint_to_accounts).with_signer(signer_seeds);

    token_interface::mint_to(cpi_ctx_mint_to, lp_token_amount)?;
    
    

    Ok(())
}
