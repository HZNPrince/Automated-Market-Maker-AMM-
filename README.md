# 🌊 Solana AMM (Automated Market Maker)

A fully functional **Automated Market Maker** built on Solana using Anchor. Implements the constant product formula (x × y = k) with token swaps, liquidity provision, and 0.3% fees.

## ✨ Features

- **Swap** - Trade tokens using constant product formula
- **Add Liquidity** - Provide liquidity and earn LP tokens
- **Remove Liquidity** - Burn LP tokens to withdraw
- **0.3% Fee** - Accrues to liquidity providers
- **Slippage Protection** - All operations include slippage checks

## 🚀 Quick Start

```bash
# Build
anchor build

# Test (requires solana-test-validator running)
anchor test --skip-local-validator
```

**All tests passing:** 5/5 ✅

## 📐 How It Works

### Constant Product Formula

```
x × y = k
```

When you swap, reserves adjust but the product stays constant.

### Fee Example

```
Swap 100 SOL → USDC
- Fee: 0.3 SOL stays in pool
- Effective input: 99.7 SOL
- Output calculated from: (reserve_a + 99.7) × (reserve_b - output) = k
```

### LP Tokens

```
Initial: sqrt(amount_a × amount_b)
Adding:  min(amount_a/reserve_a, amount_b/reserve_b) × lp_supply
```

## 📁 Structure

```
programs/amm/src/
├── instructions/
│   ├── initialize_pool.rs
│   ├── swap.rs
│   ├── add_liquidity.rs
│   └── remove_liquidity.rs
├── state.rs
└── errors.rs
```

## 📖 What I Learned

Building this taught me the fundamentals of DeFi and blockchain development:

**AMM Mechanics**

- How x×y=k powers billions in DeFi
- Why LPs earn fees (they accrue in the pool!)
- Price impact from larger trades

**Blockchain Math**

- Why `f64` doesn't work (non-deterministic)
- Integer arithmetic with `u128` and `checked_mul/div`
- Custom integer square root for determinism

**Security**

- PDAs for vault control (no private keys!)
- `transfer_checked` to validate decimals
- Slippage protection against frontrunning

**Smart Designs**

- Proportional deposits (only transfer what's needed)
- Reading reserves BEFORE transfers
- Fees staying in pool benefit all LPs

**Aha Moments** 💡

- PDAs are magical - deterministic addresses without private keys
- Same user gets fees back if they own all LP tokens
- Even tiny math differences break blockchain consensus

This project taught me not just _how_ to build an AMM, but _why_ DeFi works the way it does.

## 🎯 Next Steps

- Frontend dapp (Vite + React)
- Multi-wallet support
- Pool analytics

---

**Built with:** [Anchor](https://anchor-lang.com) | [Solana](https://solana.com)
