use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};
use crate::state::{Escrow};
use crate::{constants::*, escrow};


#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct MakeOffer<'info> {

#[account(mut)]
pub maker: Signer<'info>,

pub token_mint_a: InterfaceAccount<'info, Mint>,
pub token_mint_b: InterfaceAccount<'info, Mint>,    

#[account(
  mut,
  associated_token::mint = token_mint_a,
  associated_token::authority = maker,
 // associated_token::token_program = token_program,   //not needed because anchor under the hood knows how to get the token program
)]
pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

#[account(
  init, 
  payer = maker, 
  space = 8 + Escrow::INIT_SPACE,
  seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],  
  bump
)]
pub escrow: Account<'info, Escrow>,

#[account(
  init,
  payer = maker,
  associated_token::mint = token_mint_a,
  associated_token::authority = escrow,  
  // associated_token::token_program = token_program,   //not needed because anchor under the hood knows how to get the token program
)]
pub vault: InterfaceAccount<'info, TokenAccount>, //could name it escrow_token_account

pub token_program: Interface<'info, TokenInterface>,
pub associated_token_program: Program<'info, AssociatedToken>,
pub system_program: Program<'info, System>,
}




pub fn init_escrow(ctx: Context<MakeOffer>, seed: u64, receive: u64) -> Result<()> {
    ctx.accounts.escrow.set_inner(Escrow {
        seed,
        maker: ctx.accounts.maker.key(),
        token_mint_a: ctx.accounts.token_mint_a.key(),
        token_mint_b: ctx.accounts.token_mint_b.key(),
        receive_amount: receive,
        bump: ctx.bumps.escrow,
    });
    Ok(())
}

//here we wont use the escrow state info because we need to make the transfer first
//thats why we use the deposit argument, otherwise we could have used the escrow state info

pub fn deposit(ctx: &Context<MakeOffer>, deposit: u64) -> Result<()> {
    let maker = &ctx.accounts.maker;
    let vault = &ctx.accounts.vault;
    let maker_token_account_a = &ctx.accounts.maker_token_account_a;
    let token_program = &ctx.accounts.token_program;
    let mint = &ctx.accounts.token_mint_a;


    // Transfer tokens from maker to escrow
    let cpi_accounts = TransferChecked {
        from: maker_token_account_a.to_account_info(),
        to: vault.to_account_info(),
        authority: maker.to_account_info(),
        mint: mint.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, deposit, mint.decimals)?;

    Ok(())
}