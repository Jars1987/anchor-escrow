use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, CloseAccount, close_account};
use crate::state::{Escrow};


#[derive(Accounts)]
pub struct RefundOffer<'info> {

  #[account(mut)]
  pub maker: Signer<'info>,

  #[account(
    address = escrow.token_mint_a
  )]
  pub token_mint_a: InterfaceAccount<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = token_mint_a,
    associated_token::authority = maker,
  )]
  pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

  #[account(
    mut,
    seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
    bump = escrow.bump,
    close = maker,
    has_one = maker,
  )]
  pub escrow: Account<'info, Escrow>,

  #[account(
    mut,
    associated_token::mint = token_mint_a,
    associated_token::authority = escrow,
  )]
  pub vault: InterfaceAccount<'info, TokenAccount>,

  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
}

impl <'info> RefundOffer<'info> {
  pub fn withdraw_and_close_vault(&self) -> Result<()> {

    let seed = self.escrow.seed.to_le_bytes();
    let bump = self.escrow.bump;

    let seeds = &[b"escrow", self.maker.to_account_info().key.as_ref(), seed.as_ref(), &[bump]];
    let signer_seeds = &[&seeds[..]];


   let cpi_accounts = TransferChecked {
      from: self.vault.to_account_info(),
      mint: self.token_mint_a.to_account_info(),
      to: self.maker_token_account_a.to_account_info(),
      authority: self.escrow.to_account_info(),
    };
    let cpi_program = self.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer_checked(cpi_ctx, self.vault.amount, self.token_mint_a.decimals)?;

    let new_cpi_accounts = CloseAccount {
      account: self.vault.to_account_info(),
      destination: self.maker.to_account_info(),
      authority: self.escrow.to_account_info(),
    };
    let new_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), new_cpi_accounts, signer_seeds);

    close_account(new_cpi_ctx)?;
    Ok(())
  }

}