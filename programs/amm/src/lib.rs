use anchor_lang::prelude::*;

mod instructions;
use instructions::*;

mod errors;
mod state;

declare_id!("WZgFDnddttT4eP5AQMXAYNSqE5v8oHxJPFAWMpTdNzw");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        process_initialize_pool(ctx, token_a_amount, token_b_amount)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        process_swap(ctx, amount_in, minimum_amount_out, a_to_b)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        minimum_lp_tokens: u64,
    ) -> Result<()> {
        process_add_liquidity(ctx, amount_a, amount_b, minimum_lp_tokens)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        minimum_token_a: u64,
        minimum_token_b: u64,
    ) -> Result<()> {
        process_remove_liquidity(ctx, lp_amount, minimum_token_a, minimum_token_b)
    }
}
