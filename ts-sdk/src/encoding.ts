import { createHash } from "node:crypto";
import type { PublicKey } from "@solana/web3.js";
import type { TargetInstruction } from "./types.js";
import type { Trigger } from "./trigger.js";

export const TRIGGER_DATA_LEN = 128;
export const TARGET_IX_DATA_LEN = 512;

/// Anchor instruction discriminator for `register_job` (see idl/vigyl.json).
export const REGISTER_JOB_DISCRIMINATOR = Uint8Array.from([
  193, 91, 44, 51, 30, 133, 240, 168,
]);

const enc = new TextEncoder();

export function encodeTrigger(trigger: Trigger): {
  triggerType: number;
  triggerData: Uint8Array;
} {
  const buf = new Uint8Array(TRIGGER_DATA_LEN);
  switch (trigger.type) {
    case "cron": {
      const bytes = enc.encode(trigger.expression);
      if (bytes.length > 40) throw new Error("cron expression exceeds 40 bytes");
      buf.set(bytes, 0);
      new DataView(buf.buffer, buf.byteOffset, buf.byteLength).setBigInt64(
        40,
        BigInt(trigger.timezoneOffsetSeconds),
        true,
      );
      return { triggerType: 0, triggerData: buf };
    }
    case "account_state": {
      buf.set(trigger.watchedAccount.toBytes(), 0);
      const hash = Uint8Array.from(Buffer.from(trigger.expectedHashHex, "hex"));
      if (hash.length !== 32) throw new Error("expected hash must be 32 bytes");
      buf.set(hash, 32);
      const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
      view.setUint16(64, trigger.dataOffset, true);
      view.setUint16(66, trigger.dataLen, true);
      return { triggerType: 1, triggerData: buf };
    }
    case "price_threshold": {
      buf.set(trigger.pythFeed.toBytes(), 0);
      const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
      view.setBigInt64(32, trigger.thresholdPriceE6, true);
      view.setUint8(40, trigger.direction === "above" ? 0 : 1);
      view.setBigUint64(41, BigInt(trigger.maxConfidencePct), true);
      view.setBigInt64(49, BigInt(trigger.minPublishTimeSeconds), true);
      return { triggerType: 2, triggerData: buf };
    }
    case "slot_epoch": {
      const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
      view.setUint8(0, trigger.granularity === "slot" ? 0 : 1);
      view.setBigUint64(1, BigInt(trigger.periodSlots), true);
      return { triggerType: 3, triggerData: buf };
    }
  }
}

export function padInstructionData(ixData: Uint8Array): Uint8Array {
  if (ixData.length > TARGET_IX_DATA_LEN - 2) {
    throw new Error(`ixData exceeds ${TARGET_IX_DATA_LEN - 2} bytes`);
  }
  const buf = new Uint8Array(TARGET_IX_DATA_LEN);
  new DataView(buf.buffer, buf.byteOffset, buf.byteLength).setUint16(0, ixData.length, true);
  buf.set(ixData, 2);
  return buf;
}

/**
 * Hash the target account list the same way the program verifies it:
 * `sha256(concat(pubkey || is_signer || is_writable))` in supply order.
 */
export function hashTargetAccounts(accounts: TargetInstruction["accounts"]): Uint8Array {
  const hasher = createHash("sha256");
  for (const acct of accounts) {
    hasher.update(Buffer.from(acct.pubkey.toBytes()));
    hasher.update(Buffer.from([acct.isSigner ? 1 : 0, acct.isWritable ? 1 : 0]));
  }
  return Uint8Array.from(hasher.digest());
}

/**
 * Borsh-encode the `register_job` argument tuple, prefixed with the Anchor
 * discriminator. The byte layout matches idl/vigyl.json exactly.
 */
export function encodeRegisterJobData(args: {
  triggerType: number;
  triggerData: Uint8Array;
  targetProgram: PublicKey;
  targetIxData: Uint8Array;
  targetAccountsHash: Uint8Array;
  budgetLamports: bigint;
  maxPriorityFeeMicroLamports: bigint;
}): Uint8Array {
  const len = 8 + 1 + TRIGGER_DATA_LEN + 32 + TARGET_IX_DATA_LEN + 32 + 8 + 8;
  const buf = new Uint8Array(len);
  const view = new DataView(buf.buffer);
  let o = 0;
  buf.set(REGISTER_JOB_DISCRIMINATOR, o);
  o += 8;
  view.setUint8(o, args.triggerType);
  o += 1;
  buf.set(args.triggerData, o);
  o += TRIGGER_DATA_LEN;
  buf.set(args.targetProgram.toBytes(), o);
  o += 32;
  buf.set(args.targetIxData, o);
  o += TARGET_IX_DATA_LEN;
  buf.set(args.targetAccountsHash, o);
  o += 32;
  view.setBigUint64(o, args.budgetLamports, true);
  o += 8;
  view.setBigUint64(o, args.maxPriorityFeeMicroLamports, true);
  return buf;
}
