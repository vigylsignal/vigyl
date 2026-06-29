# Job spec

VIGYL jobs are declared as JSON off-chain, then packed into the 128-byte `trigger_data` buffer on-chain. This document defines the JSON schema and the byte layout.

## JSON schema

```json
{
  "version": 1,
  "trigger": { "type": "cron | account_state | price_threshold | slot_epoch" },
  "target": {
    "program": "<base58 program id>",
    "instruction": {
      "data_base64": "<base64 encoded instruction data>",
      "discriminator": "<optional 8-byte hex, for reference>"
    },
    "accounts": [
      { "pubkey": "<base58>", "is_signer": false, "is_writable": true }
    ]
  },
  "budget": {
    "initial_lamports": 50000000,
    "max_priority_fee_micro_lamports": 5000
  },
  "policy": {
    "max_failures_before_pause": 3,
    "retry_backoff_slots": [10, 60, 300]
  }
}
```

## Cron

```json
{
  "type": "cron",
  "expression": "0 * * * *",
  "timezone_offset_seconds": 0
}
```

- Expression up to 40 ASCII characters (on-chain fixed buffer).
- 5-field Vixie / POSIX cron. Ranges, lists, and steps supported.
- `timezone_offset_seconds` is applied by the daemon before evaluating.

Byte layout (bytes 0..128):

```
[0..40]   cron expression, zero-padded UTF-8
[40..48]  i64 LE timezone offset seconds
[48..128] reserved (zero)
```

## Account state

```json
{
  "type": "account_state",
  "watched_account": "<base58>",
  "data_offset": 8,
  "data_len": 32,
  "expected_hash_hex": "<64-char hex, sha256 of last observed slice>"
}
```

- `data_len == 0` hashes the entire account data.
- The trigger fires whenever the recomputed hash differs from `expected_hash_hex`.

Byte layout:

```
[0..32]   watched_account pubkey
[32..64]  expected_hash (32 bytes)
[64..66]  u16 LE data_offset
[66..68]  u16 LE data_len
[68..128] reserved
```

## Price threshold (Pyth)

```json
{
  "type": "price_threshold",
  "pyth_feed": "<base58>",
  "threshold_price_e6": 200000000,
  "direction": "above",
  "max_confidence_pct": 100,
  "min_publish_time_seconds": 60
}
```

- `threshold_price_e6 = price * 1e6` (so `$200` for SOL becomes `200_000_000`).
- Fires when a fresh Pyth pull update crosses the threshold in the requested direction, with a confidence interval below `max_confidence_pct`.

Byte layout:

```
[0..32]   pyth_feed pubkey
[32..40]  i64 LE threshold_price_e6
[40]      u8 direction (0 = above, 1 = below)
[41..49]  u64 LE max_confidence_pct
[49..57]  i64 LE min_publish_time_seconds
[57..128] reserved
```

## Slot / epoch

```json
{ "type": "slot_epoch", "granularity": "slot", "period_slots": 100 }
```

or

```json
{ "type": "slot_epoch", "granularity": "epoch" }
```

Byte layout:

```
[0]       u8 granularity (0 = slot, 1 = epoch)
[1..9]    u64 LE period_slots (granularity=slot)
[9..17]   u64 LE last_fired_slot
[17..128] reserved
```

## Target instruction

The target instruction is packed into `target_ix_data: [u8; 512]` with a u16 LE length prefix at bytes 0..2. Accounts hash to `target_accounts_hash: [u8; 32]` via `sha256(concat(pubkey || flags))`, iterated in the same order the daemon supplies at execution time.
