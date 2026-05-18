
use {
    anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas, solana_program::{instruction::Instruction, msg}, system_program::ID as SYSTEM_PROGRAM_ID}, litesvm::LiteSVM, solana_keypair::Keypair, solana_message::{Message}, solana_pubkey::Pubkey, solana_signer::Signer, solana_transaction::{Transaction}
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = anchor_vault::id();  // address of the vault program
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap(); 
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();  // airdrop payer 10 sol 

    (svm, payer)
}

#[test]
fn test_initialize_deposit_withdraw_close() {

    // setup 
    let (mut svm, payer) = setup();

    let user = payer.pubkey();

    let (vault_state_pda, state_bump) = 
        Pubkey::find_program_address(&[b"state", user.as_ref()], &anchor_vault::id());

    let (vault_pda, vault_bump) = 
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &anchor_vault::id());

    // Initialize
    let init_ix = Instruction{
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Initialize{
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID
        }.to_account_metas(None),
        data: anchor_vault::instruction::Initialize {}.data()
    };

    // build and send the transaction
    let message = Message::new(&[init_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let tx1 = svm.send_transaction(transaction).unwrap();

    msg!("Initialize transaction successful");
    msg!("Tx Signature: {}", tx1.signature);

    // Assert
    let vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let vault_state = 
        anchor_vault::state::VaultState::try_deserialize(&mut vault_state_account.data.as_ref()).unwrap();
    
    assert_eq!(vault_state.vault_bump, vault_bump);
    assert_eq!(vault_state.state_bump, state_bump);


    // Deposit
    let deposit_amount: u64 = 1_000_000_000;

    let deposit_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Deposit {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID
        }.to_account_metas(None),
        data: anchor_vault::instruction::Deposit {
            amount: deposit_amount
        }.data()
    };

    // build and send the transaction
    let message = Message::new(&[deposit_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let deposit_transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let deposit_tx = svm.send_transaction(deposit_transaction).unwrap();

    msg!("Deposit transaction successful");
    msg!("Deposit Tx Signature: {}", deposit_tx.signature);

    let vault_balance_after_deposit = svm.get_balance(&vault_pda).unwrap();

    // assert
    assert_eq!(vault_balance_after_deposit, deposit_amount);
    msg!("{} deposited into account successfully", deposit_amount);


    // Withdraw
    let withdraw_amount: u64 = deposit_amount / 2;

    let withdraw_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Withdraw {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID
        }.to_account_metas(None),
        data: anchor_vault::instruction::Withdraw {
            amount: withdraw_amount
        }.data()
    };

    // build and send the transaction
    let message = Message::new(&[withdraw_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let withdraw_transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let withdraw_tx = svm.send_transaction(withdraw_transaction).unwrap();

    msg!("Withdraw transaction successful");
    msg!("Withdraw Tx Signature: {}", withdraw_tx.signature);

    let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap();

    // assert
    assert_eq!(vault_balance_after_withdraw, withdraw_amount);
    msg!("{} withdrawn from account successfully", withdraw_amount);



    // Close account

    let close_amount = svm.get_balance(&vault_pda).unwrap();

    let close_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Close {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID
        }.to_account_metas(None),
        data: anchor_vault::instruction::Close {}.data()
    };

    // build and send the transaction
    let message = Message::new(&[close_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let close_transaction = Transaction::new(&[&payer], message, recent_blockhash);
    let close_tx = svm.send_transaction(close_transaction).unwrap();

    msg!("Close transaction successful");
    msg!("Close Tx Signature: {}", close_tx.signature);

    // assert
    assert!(svm.get_account(&vault_pda).is_none()); 
    assert!(svm.get_account(&vault_state_pda).is_none());
    let user_balance_after_close = svm.get_balance(&user).unwrap();
    assert!(user_balance_after_close > close_amount);
    msg!("Balance after close: {}", user_balance_after_close);

}

// #[test]
// fn test_init_and_deposit_more_than_in_user_account(){

//     let (mut svm, payer) = setup();
//     let user = payer.pubkey();
//     let (vault_state_pda, state_bump) = Pubkey::find_program_address(seeds, program_id)

// }