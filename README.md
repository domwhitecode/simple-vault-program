# anchor-vault

A minimal Solana vault built with Anchor. Each user can initialize a personal vault, deposit SOL into it, withdraw, and close the vault to reclaim everything.

Program ID: `8d8aW4TGCQhzvkWkTs8nGM4XdjF36yY7Kwz13Sbm2g4g`

## How it works

Each user gets two on-chain accounts, both PDAs derived from their wallet:

| Account       | Seeds                                  | Purpose                                                |
| ------------- | -------------------------------------- | ------------------------------------------------------ |
| `vault_state` | `[b"state", user.key()]`               | Stores the bumps for both PDAs (2 bytes + discriminator). |
| `vault`       | `[b"vault", vault_state.key()]`        | A `SystemAccount` that holds the deposited lamports.   |

The vault is a plain system-owned account — deposits and withdrawals are System Program `transfer` CPIs. The vault signs withdrawals/closes for itself using its PDA seeds.

## Instructions

| Instruction  | Effect                                                                         |
| ------------ | ------------------------------------------------------------------------------ |
| `initialize` | Creates `vault_state` (rent paid by user). Stores both bumps on the state.     |
| `deposit`    | Transfers `amount` lamports from user → vault.                                 |
| `withdraw`   | Transfers `amount` lamports from vault → user. Vault signs via its PDA.        |
| `close`      | Drains all vault lamports back to user, then closes `vault_state` (rent refund). |

## Project layout

```
programs/anchor-vault/
├── src/
│   ├── lib.rs                # #[program] entry point
│   ├── state.rs              # VaultState (bumps)
│   ├── constants.rs          # STATE_SEED, VAULT_SEED, ONE_SOL
│   └── instructions/
│       ├── initialize.rs
│       ├── deposit.rs
│       ├── withdraw.rs
│       └── close.rs
└── tests/
    └── test_vault.rs         # LiteSVM integration tests
```

## Build & test

Tests use **LiteSVM**, which runs your program's compiled `.so` inside an in-process Solana VM. No validator, no network, no deploy.

```bash
# Build the program (produces target/deploy/anchor_vault.so)
anchor build

# Run tests
cargo test
```

### Why not `anchor test`?

`anchor test` boots a local validator (Surfpool in Anchor 0.31+), deploys the program to it, then runs the script in `[scripts] test`. That workflow is for TypeScript tests that talk to a real validator over RPC. LiteSVM loads the `.so` directly via `include_bytes!` at compile time — there's nothing to deploy and no validator to connect to. Skip `anchor test` and just run `cargo test`.

If you ever need the validator workflow (e.g., for TS integration tests), use `anchor test --skip-local-validator` against a separately-running validator.

## Tests

`tests/test_vault.rs` covers:

- Initialize stores the correct bumps on `vault_state`
- Full happy path: init → deposit → withdraw → close
- Deposit larger than user's balance fails (and vault stays untouched)
- Multiple deposits accumulate correctly
- Withdraw larger than vault balance fails
- Re-initializing an existing vault fails
- A different user cannot withdraw from someone else's vault (seed-binding security check)
- Close returns vault funds + state rent back to the user

Tests share a small helper layer (`initialize_ix`, `deposit_ix`, `send`, etc.) so each test stays focused on what it's asserting.
