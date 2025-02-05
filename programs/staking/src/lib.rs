use anchor_lang::prelude::*;

declare_id!("7D9dojuMGTbs9FjrRfuuaK9NZMMcrhqVHyYwnyfmJwMm");

pub mod contexts;
pub mod error;
pub mod state;

#[program]
pub mod staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
