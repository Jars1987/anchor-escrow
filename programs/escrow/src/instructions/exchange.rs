use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, CloseAccount, close_account};
use crate::state::{Escrow};



#[derive(Accounts)]
pub struct TakeOffer<'info> {
  
#[account(mut)]
pub taker: Signer<'info>,

#[account(
  address = escrow.maker,
)]
pub maker: SystemAccount<'info>,

#[account(
  address = escrow.token_mint_a
)]
pub token_mint_a: InterfaceAccount<'info, Mint>,
#[account(
  address = escrow.token_mint_b
)]
pub token_mint_b: InterfaceAccount<'info, Mint>,

#[account(
  init_if_needed,
  payer = taker,
    associated_token::mint = token_mint_a,
    associated_token::authority = taker,
)]
pub taker_token_account_a: InterfaceAccount<'info, TokenAccount>,

#[account(
    mut,
    associated_token::mint = token_mint_b,
    associated_token::authority = taker,
)]
pub taker_token_account_b: InterfaceAccount<'info, TokenAccount>,

#[account(
  init_if_needed,
  payer = taker,
  associated_token::mint = token_mint_b,
  associated_token::authority = maker,  
)]
pub maker_token_account_b: InterfaceAccount<'info, TokenAccount>,

#[account(
mut, 
has_one= token_mint_a,
has_one= token_mint_b,
seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],  
bump = escrow.bump, 
close = taker)]
pub escrow: Account<'info, Escrow>,

#[account(
  mut, 
  associated_token::mint = token_mint_a,
  associated_token::authority = escrow,  
)]
  pub vault: InterfaceAccount<'info, TokenAccount>, //could name it escrow_ATA

pub token_program: Interface<'info, TokenInterface>,
pub associated_token_program: Program<'info, AssociatedToken>,
pub system_program: Program<'info, System>,
}


impl<'info> TakeOffer<'info> {
pub fn send_wanted_tokens_to_maker(&mut self) -> Result<()> {

  // Transfer the wanted tokens from the taker to the maker
    let cpi_accounts = TransferChecked {
        from: self.taker_token_account_b.to_account_info(),
        mint: self.token_mint_b.to_account_info(),
        to: self.maker_token_account_b.to_account_info(),
        authority: self.taker.to_account_info(),
    };
    let cpi_program = self.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_ctx, self.escrow.receive_amount , self.token_mint_b.decimals)?;
    Ok(())

}

pub fn withdraw_and_close_vault(&mut self) -> Result<()> {

  let escrow = self.escrow.to_account_info();

  let cpi_accounts = TransferChecked {
    from: self.vault.to_account_info(),
    mint: self.token_mint_a.to_account_info(),
    to: self.taker_token_account_a.to_account_info(),
    authority: escrow,
};

let seed_bytes = self.escrow.seed.to_le_bytes();

let seeds = &[
  b"escrow", 
  self.escrow.maker.as_ref(),
  seed_bytes.as_ref(),
  &[self.escrow.bump]
];

let signer_seeds = [&seeds[..]];  

let cpi_program = self.token_program.to_account_info();
let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

transfer_checked(cpi_context, self.vault.amount, self.token_mint_a.decimals)?;


let accounts = CloseAccount {
  account: self.vault.to_account_info(),
  destination: self.taker.to_account_info(),
  authority: self.escrow.to_account_info(),
};

let cpi_context = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer_seeds);

close_account(cpi_context)?;

  Ok(())
}

}


//Make Refund to cancel escrow