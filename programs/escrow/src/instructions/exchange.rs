use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account}
};
use crate::state::{Escrow};



#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct TakeOffer<'info> {
#[account(mut)]
pub taker: Signer<'info>,

#[account(
 mut
)]
pub maker: SystemAccount<'info>,

pub token_mint_a: InterfaceAccount<'info, Mint>,
pub token_mint_b: InterfaceAccount<'info, Mint>,

#[account(
  init_if_needed,
  payer = taker,
  associated_token::mint = token_mint_a,
  associated_token::authority = taker,
)]
pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

#[account(
 mut,  
  associated_token::mint = token_mint_b,
  associated_token::authority = taker,
)]
pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

#[account(
  init_if_needed,
  payer = taker,
  associated_token::mint = token_mint_b,
  associated_token::authority = maker,
)]
pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

#[account(
  mut,
  close = maker,
  has_one = maker,
  has_one = token_mint_a, //the account name and the state have to match the samne name otherwise the had_one will not work constraint wont work
  has_one = token_mint_b,
  seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
  bump
)]
pub escrow: Account<'info, Escrow>,


#[account(
  mut,
  associated_token::mint = token_mint_a,
  associated_token::authority = escrow,
)]
pub vault: InterfaceAccount<'info, TokenAccount>,

pub system_program: Program<'info, System>,
pub token_program: Interface<'info, TokenInterface>,
pub associated_token_program: Program<'info, AssociatedToken>
}


//First we make a direct transder of the tokens from the taker to the maker
pub fn send_wanted_tokens_to_maker(context: &Context<TakeOffer>,) -> Result<()> {
  
  // Transfer tokens from maker to escrow
  let cpi_accounts = TransferChecked {
    from: context.accounts.taker_token_account_b.to_account_info(),
    to: context.accounts.maker_token_account_b.to_account_info(),
    authority: context.accounts.taker.to_account_info(),
    mint: context.accounts.token_mint_b.to_account_info(),
};
  let cpi_program = context.accounts.token_program.to_account_info();

  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

  transfer_checked(cpi_ctx, context.accounts.escrow.receive_amount, context.accounts.token_mint_b.decimals)?;

  Ok(())
}

//Now we make a transfer from the vault to the taker and close the vault

pub fn withdraw_and_close_vault(context: Context<TakeOffer>) -> Result<()> {
  /*
  context.accounts.escrow.seed.to_le_bytes() produces a temporary value, 
  and you are trying to reference it in the seeds array, which expects references
  with a lifetime that outlives the statement. When the temporary value is 
  dropped, the reference becomes invalid.
  
  To fix this, you need to create a binding (a variable) for the result of 
  to_le_bytes() so it has a longer lifetime, as suggested in the error message.
   */
  let seed_bytes = context.accounts.escrow.seed.to_le_bytes();
  let seeds = &[
    b"escrow", 
    context.accounts.maker.to_account_info().key.as_ref(),
    seed_bytes.as_ref(),
    &[context.accounts.escrow.bump]
];

  let signer_seeds = [&seeds[..]];

  let accounts = TransferChecked {
    from: context.accounts.vault.to_account_info(),
    to: context.accounts.taker_token_account_a.to_account_info(),
    mint: context.accounts.token_mint_a.to_account_info(),
    authority: context.accounts.escrow.to_account_info(),
  };

  let cpi_context = CpiContext::new_with_signer(context.accounts.token_program.to_account_info(), accounts, &signer_seeds);

  transfer_checked(cpi_context, context.accounts.vault.amount, context.accounts.token_mint_a.decimals)?;



  //Now we close the account so it can't be used again so we make a close acount struct to be used on another cpi 
  //Needs to be close this way because the vault is a associated token account

  let accounts = CloseAccount {
    account: context.accounts.vault.to_account_info(),
    destination: context.accounts.taker.to_account_info(),
    authority: context.accounts.escrow.to_account_info(),
  };

  let cpi_context = CpiContext::new_with_signer(context.accounts.token_program.to_account_info(), accounts, &signer_seeds);

  close_account(cpi_context)?;

  Ok(())

}


