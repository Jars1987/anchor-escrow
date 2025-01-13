use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub receive_amount: u64,
    pub bump: u8,
}

//we don't need the amount offered because we can get it from the vault: ctx.accounts.vault.amount