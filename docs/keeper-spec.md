# Keeper daemon spec

`vigyl keeper run` boots the daemon. This document describes its contract with the on-chain program.

## Boot sequence

1. Load the operator wallet (`--wallet-path` or `SOLANA_WALLET`).
2. Load config (RPC url, program id, min priority fee, max concurrent, health port).
3. Verify a `KeeperBond` PDA exists for the wallet:
   - Missing -> print `bond required, run: vigyl keeper bond <amount>` and exit.
   - Present but `bond_amount < config.min_keeper_bond` -> same message and exit.
4. Start the http health server on `--health-port` (default 9090).
5. Subscribe to program account changes (jobs, executions).
6. Enter the main loop.

## Main loop (per slot)

For every newly observed slot:

1. Fetch the active jobs (`is_paused = false`, `budget > min_run_cost`).
2. Filter those whose `next_run_slot <= current_slot`.
3. For each candidate job:
   - Compute the bond-weighted leader (`weighted_leader(candidates, min_bond, job_pubkey, slot)`).
   - If this daemon is the leader:
     - `claim_execution(expected_run_slot)`.
     - If confirmed, build the CPI to `job.target_program` with the trigger data.
     - Simulate before broadcasting.
     - Sign, broadcast, wait for confirmation.
     - `submit_proof(execution_index, tx_signature, success, priority_fee_used)`.
   - Otherwise no-op.
4. Enforce `--max-concurrent` via a semaphore around the CPI section.

## Assignment failure paths

- Another keeper claimed first -> `AlreadyAssigned` returned. Log `lost_claim` and continue.
- CPI simulate returns error -> retry once with p95 priority fee and a fresh blockhash. If still failing, submit `success = false` so `failure_count` increments and the job eventually pauses at `max_failures_before_pause`.
- Confirmation timeout -> retry once, then submit `success = false`.
- No `submit_proof` at all -> `execution_timeout_slots` expires, and any other keeper can call `slash_keeper`.

## Bond and slash flow

- `vigyl keeper bond <amount>` transfers `$VIGYL` to the program vault and updates `KeeperBond`.
- `vigyl keeper unbond` sets `is_unbonding = true`. Rejected if `active_jobs > 0`.
- `vigyl keeper withdraw` releases the bond after `config.bond_unlock_seconds`.
- `slash_keeper` splits the bond: `slash_burn_bps` burned, `slash_owner_bps` paid to the job owner, remainder stays with the keeper.

## Metrics logs

Structured log lines, one per keeper action:

```
{"level":"info","name":"vigyl-keeper","event":"claim","job":"...","result":"ok"}
{"level":"info","name":"vigyl-keeper","event":"proof","job":"...","success":true,"latency_slots":3,"priority_fee":4500}
{"level":"info","name":"vigyl-keeper","event":"lost_claim","job":"...","winner":"..."}
```

## Graceful shutdown

- SIGINT / SIGTERM -> stop accepting new claims. Wait up to 30 seconds for in-flight claims to submit proofs, then exit.
- Assignments that never received a proof will be picked up by any other keeper via `slash_keeper` once the timeout expires.
