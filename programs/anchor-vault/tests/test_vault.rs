use {
    anchor_lang::{
        AccountDeserialize, InstructionData, ToAccountMetas,
        solana_program::instruction::Instruction,
        system_program::ID as SYSTEM_PROGRAM_ID,
    },
    anchor_vault::{ONE_SOL, STATE_SEED, VAULT_SEED},
    litesvm::{
        types::{FailedTransactionMetadata, TransactionMetadata},
        LiteSVM,
    },
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

// ---------- helpers ----------

fn setup() -> (LiteSVM, Keypair) {
    let program_id = anchor_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10 * ONE_SOL).unwrap();
    (svm, payer)
}

fn state_pda(user: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[STATE_SEED, user.as_ref()], &anchor_vault::id()).0
}

fn vault_pda(user: Pubkey) -> Pubkey {
    let state = state_pda(user);
    Pubkey::find_program_address(&[VAULT_SEED, state.as_ref()], &anchor_vault::id()).0
}

fn initialize_ix(user: Pubkey) -> Instruction {
    Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Initialize {
            user,
            vault_state: state_pda(user),
            vault: vault_pda(user),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Initialize {}.data(),
    }
}

fn deposit_ix(user: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Deposit {
            user,
            vault_state: state_pda(user),
            vault: vault_pda(user),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Deposit { amount }.data(),
    }
}

fn withdraw_ix(user: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Withdraw {
            user,
            vault_state: state_pda(user),
            vault: vault_pda(user),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Withdraw { amount }.data(),
    }
}

fn close_ix(user: Pubkey) -> Instruction {
    Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Close {
            user,
            vault_state: state_pda(user),
            vault: vault_pda(user),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Close {}.data(),
    }
}

fn send(
    svm: &mut LiteSVM,
    signer: &Keypair,
    ix: Instruction,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    let message = Message::new(&[ix], Some(&signer.pubkey()));
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new(&[signer], message, blockhash);
    svm.send_transaction(tx)
}

// ---------- tests ----------

#[test]
fn test_initialize_stores_bumps() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();
    let (_, expected_state_bump) =
        Pubkey::find_program_address(&[STATE_SEED, user.as_ref()], &anchor_vault::id());
    let (_, expected_vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, state_pda(user).as_ref()], &anchor_vault::id());

    send(&mut svm, &payer, initialize_ix(user)).unwrap();

    let acct = svm.get_account(&state_pda(user)).unwrap();
    let vault_state =
        anchor_vault::state::VaultState::try_deserialize(&mut acct.data.as_ref()).unwrap();
    assert_eq!(vault_state.state_bump, expected_state_bump);
    assert_eq!(vault_state.vault_bump, expected_vault_bump);
}

#[test]
fn test_full_flow_init_deposit_withdraw_close() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();
    let vault = vault_pda(user);

    send(&mut svm, &payer, initialize_ix(user)).unwrap();

    send(&mut svm, &payer, deposit_ix(user, ONE_SOL)).unwrap();
    assert_eq!(svm.get_balance(&vault).unwrap(), ONE_SOL);

    send(&mut svm, &payer, withdraw_ix(user, ONE_SOL / 2)).unwrap();
    assert_eq!(svm.get_balance(&vault).unwrap(), ONE_SOL / 2);

    send(&mut svm, &payer, close_ix(user)).unwrap();
    assert!(svm.get_account(&state_pda(user)).is_none());
    assert_eq!(svm.get_balance(&vault).unwrap_or(0), 0);
}

#[test]
fn test_deposit_more_than_balance_fails() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();
    let vault = vault_pda(user);

    send(&mut svm, &payer, initialize_ix(user)).unwrap();

    let err = send(&mut svm, &payer, deposit_ix(user, 11 * ONE_SOL)).unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("insufficient lamports")),
        "expected insufficient-lamports failure, got: {:#?}",
        err.meta.logs
    );
    assert_eq!(
        svm.get_balance(&vault).unwrap_or(0),
        0,
        "vault should be untouched"
    );
}

#[test]
fn test_multiple_deposits_accumulate() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();

    send(&mut svm, &payer, initialize_ix(user)).unwrap();
    send(&mut svm, &payer, deposit_ix(user, ONE_SOL)).unwrap();
    send(&mut svm, &payer, deposit_ix(user, 2 * ONE_SOL)).unwrap();

    assert_eq!(svm.get_balance(&vault_pda(user)).unwrap(), 3 * ONE_SOL);
}

#[test]
fn test_withdraw_more_than_vault_balance_fails() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();
    let vault = vault_pda(user);

    send(&mut svm, &payer, initialize_ix(user)).unwrap();
    send(&mut svm, &payer, deposit_ix(user, ONE_SOL)).unwrap();

    let err = send(&mut svm, &payer, withdraw_ix(user, 2 * ONE_SOL)).unwrap_err();
    assert!(
        err.meta.logs.iter().any(|l| l.contains("insufficient lamports")),
        "expected insufficient-lamports failure, got: {:#?}",
        err.meta.logs
    );
    assert_eq!(svm.get_balance(&vault).unwrap(), ONE_SOL, "vault should be untouched");
}

#[test]
fn test_reinitialize_same_user_fails() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();

    send(&mut svm, &payer, initialize_ix(user)).unwrap();
    let result = send(&mut svm, &payer, initialize_ix(user));
    assert!(
        result.is_err(),
        "second initialize should fail because the vault_state PDA already exists"
    );
}

#[test]
fn test_other_user_cannot_withdraw_from_someones_vault() {
    let (mut svm, alice) = setup();
    let bob = Keypair::new();
    svm.airdrop(&bob.pubkey(), 10 * ONE_SOL).unwrap();

    send(&mut svm, &alice, initialize_ix(alice.pubkey())).unwrap();
    send(&mut svm, &alice, deposit_ix(alice.pubkey(), 5 * ONE_SOL)).unwrap();

    // Bob signs but supplies Alice's PDAs. The Withdraw constraint
    // seeds = [STATE_SEED, user.key()] uses the signer's pubkey, so the
    // seed check fails before any lamports move.
    let malicious_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Withdraw {
            user: bob.pubkey(),
            vault_state: state_pda(alice.pubkey()),
            vault: vault_pda(alice.pubkey()),
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Withdraw { amount: ONE_SOL }.data(),
    };

    let result = send(&mut svm, &bob, malicious_ix);
    assert!(result.is_err(), "bob should not be able to withdraw from alice's vault");
    assert_eq!(
        svm.get_balance(&vault_pda(alice.pubkey())).unwrap(),
        5 * ONE_SOL,
        "alice's vault should be untouched"
    );
}

#[test]
fn test_close_returns_all_funds_to_user() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();

    send(&mut svm, &payer, initialize_ix(user)).unwrap();
    send(&mut svm, &payer, deposit_ix(user, 2 * ONE_SOL)).unwrap();

    let user_before = svm.get_balance(&user).unwrap();
    send(&mut svm, &payer, close_ix(user)).unwrap();
    let user_after = svm.get_balance(&user).unwrap();

    // User reclaims the 2 SOL in the vault plus the rent that was held by
    // the vault_state account, minus a small tx fee.
    assert!(
        user_after > user_before + 2 * ONE_SOL,
        "user should reclaim vault + state rent (before={}, after={})",
        user_before,
        user_after
    );
    assert!(svm.get_account(&state_pda(user)).is_none());
    assert_eq!(svm.get_balance(&vault_pda(user)).unwrap_or(0), 0);
}
