use anchor_lang::prelude::*;
use crate::{STATE_SEED, VAULT_SEED, state::VaultState};

// all accounts needed when an instruction is invoked 
#[derive(Accounts)]
pub struct Initialize<'info> {
    
    // user account needs to be mutable because lamports state will change 
    // deposits/withdrawals 
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer=user,
        seeds = [STATE_SEED, user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE, // 8 + vault space to account for discriminator
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [VAULT_SEED, vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

// handler for initialization
impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // save data to state
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;

        Ok(())
    }
}
  