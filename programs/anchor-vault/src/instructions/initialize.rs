use anchor_lang::prelude::*;
use crate::state::VaultState;

// all accounts needed when an instructions is invoked 
#[derive(Accounts)]
pub struct Initialize<'info> {
    
    #[account(mut)] // user account needs to be mutable because lamports state will change 
    pub user: Signer<'info>,

    
    #[account(
        init,
        payer=user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
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
  