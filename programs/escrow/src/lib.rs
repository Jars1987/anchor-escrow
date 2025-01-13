pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
pub use instructions::*;



declare_id!("7K4y28DMQaREFEHy8yCGEjzFNSVTynpk8erXN5ivowGw");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<MakeOffer>, seed:u64, receive: u64, deposit: u64) -> Result<()> {

        /* 
        the reason we are calling the make::deposit function first is because
         we can't &mut the context for init_escrow 
         the accounts are already initiated so no harm in saving teh data after the deposit

         the instructions are divided in deposit and init_escrow because it's too big to fit in 
         one function
         */
        instructions::make::deposit(&ctx, deposit)?;
        instructions::make::init_escrow(ctx, seed, receive)?;
        Ok(())
    }

    pub fn exchange(ctx: Context<TakeOffer>) -> Result<()> {
        instructions::exchange::send_wanted_tokens_to_maker(&ctx)?;
        instructions::exchange::withdraw_and_close_vault(ctx)?;
        Ok(())
    }
}


