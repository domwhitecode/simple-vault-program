use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

impl<'info>Withdraw<'info> {
    // transfer money FROM: vault -> TO: user
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        let cpi_accounts = Transfer{
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        
        // we need the vault to sign for itself and therefore must 
        // derive a PDA for the vault to sign the transaction
        let seeds =  &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(System::id(), cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;
        Ok(())
    }
} 