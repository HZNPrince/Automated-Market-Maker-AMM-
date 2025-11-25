import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { Amm } from "../target/types/amm"
import {
  mintTo,
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token"
import { PublicKey } from "@solana/web3.js"

describe("amm", () => {
  // Set the Provider and Program
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.amm as Program<Amm>
  const connection = provider.connection
  const user = provider.wallet.payer

  // Declare variables
  let solMint: PublicKey
  let usdcMint: PublicKey
  let userSolAccount: PublicKey
  let userUsdcAccount: PublicKey
  let poolAddr: PublicKey
  let vaultAddrA: PublicKey
  let vaultAddrB: PublicKey
  let lpMint: PublicKey
  let userLpAccount: PublicKey

  async function logUserStats(label: string) {
    const sol = await getAccount(connection, userSolAccount)
    const usdc = await getAccount(connection, userUsdcAccount)
    const lp = await getAccount(connection, userLpAccount)

    console.log(`\n\n 📊 ${label} :`)
    console.log(`     SOL: ${Number(sol.amount) / 10 ** 6}`)
    console.log(`     USDC: ${Number(usdc.amount) / 10 ** 6}`)
    console.log(`     LP_tokens alloted: ${Number(lp.amount) / 10 ** 6}`)
  }

  async function logPoolState() {
    const vault_a = await getAccount(connection, vaultAddrA)
    const vault_b = await getAccount(connection, vaultAddrB)

    console.log(`\n\n🐳 Pool State:`)
    console.log(`     Vault: SOL  ${Number(vault_a.amount) / 10 ** 6}`)
    console.log(`     Vault: USDC  ${Number(vault_b.amount) / 10 ** 6}`)
  }

  before("Tokens and Funds setup", async () => {
    // Todo
    // Create Mint accounts
    solMint = await createMint(connection, user, user.publicKey, null, 6)
    console.log("SOL Mint Account Created: ", solMint)

    usdcMint = await createMint(connection, user, user.publicKey, null, 6)
    console.log("USDC Mint Account Created: ", usdcMint)

    // Create User Token Accounts
    userSolAccount = await createAssociatedTokenAccount(connection, user, solMint, user.publicKey)
    console.log("\n\nATA of SOL  for User created: ", userSolAccount)

    userUsdcAccount = await createAssociatedTokenAccount(connection, user, usdcMint, user.publicKey)
    console.log("ATA of USDC for User created: ", userUsdcAccount)

    // Mint Tokens to User Token Accounts
    await mintTo(connection, user, solMint, userSolAccount, user, 10000 * 10 ** 6)
    console.log("\n\n10,000 SOL minted to User")
    await mintTo(connection, user, usdcMint, userUsdcAccount, user, 10000 * 10 ** 6)
    console.log("10_000 USDC minted to User")

    // Derive PDA's for Pools, Vaults, LP_tokens
    ;[poolAddr] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), solMint.toBuffer(), usdcMint.toBuffer()],
      program.programId
    )
    console.log(`\n\nPool Address Derived: ${poolAddr}`)
    ;[vaultAddrA] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_a"), solMint.toBuffer(), usdcMint.toBuffer()],
      program.programId
    )
    ;[vaultAddrB] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_b"), solMint.toBuffer(), usdcMint.toBuffer()],
      program.programId
    )
    ;[lpMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp_mint"), solMint.toBuffer(), usdcMint.toBuffer()],
      program.programId
    )
    userLpAccount = await getAssociatedTokenAddressSync(lpMint, user.publicKey)
    console.log("ATA of LP_token for User created: ", userUsdcAccount)

    console.log("PDA's Derived! ")
  })

  it("Initialize Pool", async () => {
    // Add your test here. Todo
    const initialLiquiditySOL = new anchor.BN(1000 * 10 ** 6)
    const initialLiquidityUSDC = new anchor.BN(5000 * 10 ** 6)

    // Call Method
    const initializePoolTx = await program.methods
      .initializePool(initialLiquiditySOL, initialLiquidityUSDC)
      .accounts({
        creator: user.publicKey,
        tokenAMint: solMint,
        tokenBMint: usdcMint,
        creatorTokenA: userSolAccount,
        creatorTokenB: userUsdcAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" })

    console.log("Pool Intialized: ", initializePoolTx)

    await logUserStats("After")
    await logPoolState()
  })

  it("User Swaps token from (SOL -> USDC)", async () => {
    console.log("\n\n\n Test : Swap SOL -> USDC")
    const solToSwap = new anchor.BN(100 * 10 ** 6)
    const minimumUSDCTolerance = new anchor.BN(450 * 10 ** 6)

    // Call Method
    const swapAToBTx = await program.methods
      .swap(solToSwap, minimumUSDCTolerance, true)
      .accounts({
        signer: user.publicKey,
        tokenAMint: solMint,
        tokenBMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" })

    console.log("\nSwap Complete: ", swapAToBTx)

    await logUserStats("After Swap")
    await logPoolState()
  })

  it("User Swaps token from (USDC -> SOL)", async () => {
    console.log("\n\n\n Test : Swap USDC -> SOL")
    const usdcToSwap = new anchor.BN(1000 * 10 ** 6)
    const minimumSOLTolerance = new anchor.BN(150 * 10 ** 6)

    // Call Method
    const swapAToBTx = await program.methods
      .swap(usdcToSwap, minimumSOLTolerance, false)
      .accounts({
        signer: user.publicKey,
        tokenAMint: solMint,
        tokenBMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" })

    console.log("\nSwap Complete: ", swapAToBTx)

    await logUserStats("After Swap")
    await logPoolState()
  })

  it("User Provides Liquidity to the Pool", async () => {
    console.log("\n\n\n Test : Liquidity provider provides Liquidation to the pool")
    const solToProvide = new anchor.BN(1000 * 10 ** 6)
    const usdcToProvide = new anchor.BN(1000 * 10 ** 6)

    // Call Method
    const addLiquidityTx = await program.methods
      .addLiquidity(solToProvide, usdcToProvide, new anchor.BN(350 * 10 ** 6))
      .accounts({
        liquidityProvider: user.publicKey,
        tokenAMint: solMint,
        tokenBMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" })

    await logUserStats("After Swap")
    await logPoolState()
  })

  it("Revoking Liquidity from the pool", async () => {
    console.log("\n\n\n Test : Liquidity provider revokes Liquidation from the pool")
    const lpTokenMinted = await getAccount(connection, userLpAccount)
    console.log("LP Tokens minted to user : ", lpTokenMinted.amount)

    const removeLiquidityTx = await program.methods
      .removeLiquidity(new anchor.BN(lpTokenMinted.amount), new anchor.BN(1), new anchor.BN(1))
      .accounts({
        liquidityRevoker: user.publicKey,
        tokenAMint: solMint,
        tokenBMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" })

    await logUserStats("After Swap")
    await logPoolState()
  })
})
