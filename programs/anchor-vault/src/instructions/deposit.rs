use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Deposit<'info> {
    // both user and vault need to be mutable as lamport state will inherently change
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump // use bump saved on state

    )]
    pub vault: SystemAccount<'info>, 

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    system_program: Program<'info, System> // needed to performa transfers 
}

impl<'info>Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        let cpi_accounts = Transfer{
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(System::id(), cpi_accounts);

        transfer(cpi_ctx, amount)?;
        Ok(())
    }
} 