use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Slippage has exceeded the standard tolerance")]
    SlippageExceeded,
    #[msg("The vault accounts for the swap dont not have enough liquidity")]
    InsufficientLiquidity,
    #[msg("Please enter a valid amount for swap")]
    InvalidInput,
    #[msg("Amount recieved has become less than the minimum lp token provided")]
    SlippageExceededForLP,
    #[msg("Amount Provided is insufficient")]
    InsufficientAmount,
    #[msg("You have not contributed in this liquidity pool")]
    ZeroContriInPool,
    #[msg("Amount recieved has become less than the minimum tokens requirement")]
    SlippageExceededForLR,
}
