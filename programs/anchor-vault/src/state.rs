use anchor_lang::prelude::*;

// we can use these bump later to derive PDA's 
#[derive(InitSpace)] // will derive the space needed for rent exemption
#[account]
pub struct VaultState {
    pub vault_bump: u8, // one bump is equal to one byte hence u8
    pub state_bump: u8
}