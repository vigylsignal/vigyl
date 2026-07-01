# Security notes

The threats VIGYL defends against and how the defence lands in code.

## Fabricated proofs

*Threat:* an unbonded caller submits `submit_proof` with a made-up `tx_signature`.

*Defence:* the program requires `signer == job.assigned_keeper`. The off-chain indexer additionally verifies `tx_signature` via `getSignatureStatuses`; mismatches surface in `/executions` metadata but the on-chain proof PDA is authoritative.

## Slashing before timeout

*Threat:* a malicious caller invokes `slash_keeper` before the assigned keeper has had a chance to submit.

*Defence:* `slash_keeper` requires `clock.slot > job.assigned_at_slot + config.execution_timeout_slots`. The slot is read from the Sysvar clock, not from user input.

## Overflow / underflow

Every counter increment uses `checked_*`. Every decrement uses `saturating_sub`. See `src/job.rs::JobBudget` for the pattern.

## Reentrancy

Solana's runtime is not reentrant by default. The program keeps its state mutations before token CPIs to avoid ordering surprises when audit tools inspect the linear execution.

## Priority fee cap abuse

`submit_proof` includes `priority_fee_used`. The program rejects proofs where `priority_fee_used > job.max_priority_fee_micro_lamports`. Cap enforcement is a hard error, not a warning.

## Griefing via drained budgets

If the job budget drops below the estimated cost of a single execution, the program flips `is_paused = true`. A keeper cannot brick a job by burning budget on failing executions -- `submit_proof(success=false)` only debits the priority fee, not the notional slot.

## Sybil bonding

`bond_keeper(0)` is rejected. `request_unbond` requires `active_jobs == 0`. `withdraw_unbond` requires `config.bond_unlock_seconds` to have elapsed. This mirrors Keep3r's bond delay and prevents cheap Sybil rotation.

## RPC secret exposure

All private RPC providers (Helius, QuickNode) belong on the server. The web client bundles only `NEXT_PUBLIC_SOLANA_RPC = api.mainnet-beta.solana.com`. The build pipeline greps `.next/` for provider host substrings and fails on non-zero matches.

## Governance

Config parameter updates require `has_one = authority` and a single-signer authority. A multisig wrapper is a natural next step but is not required for the initial launch.
