use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked};
use crate::state::{Escrow};


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



impl<'info> MakeOffer<'info> {
  pub fn init_escrow(&mut self, seed: u64, receive: u64, bumps: MakeOfferBumps) -> Result<()> {
    self.escrow.set_inner(Escrow {
        seed,
        maker: self.maker.key(),
        token_mint_a: self.token_mint_a.key(),
        token_mint_b: self.token_mint_b.key(),
        receive_amount: receive,
        bump: bumps.escrow,
    });

    msg!("Escrow account created");
    msg!("Escrow account: {:?}", self.escrow.key());
    msg!("token_mint_a: {:?}", self.token_mint_a.key());
    msg!("token_mint_b: {:?}", self.token_mint_b.key());
    msg!("maker_token_account_a: {:?}", self.maker_token_account_a.key());
    msg!("vault: {:?}", self.vault.key());
    msg!("-----------------------------------------------------------------");

    Ok(())
}

pub fn deposit(&mut self, deposit: u64) -> Result<()> {


  // Transfer tokens from maker to escrow
  let cpi_accounts = TransferChecked {
      from: self.maker_token_account_a.to_account_info(),
      to: self.vault.to_account_info(),
      authority: self.maker.to_account_info(),
      mint: self.token_mint_a.to_account_info(),
  };
  let cpi_program = self.token_program.to_account_info();
  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
  transfer_checked(cpi_ctx, deposit, self.token_mint_a.decimals)?;

  Ok(())
}
}

