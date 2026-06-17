import { describe, expect, it } from "vitest";
import { PublicKey } from "@solana/web3.js";
import {
  TARGET_IX_DATA_LEN,
  TRIGGER_DATA_LEN,
  encodeRegisterJobData,
  encodeTrigger,
  hashTargetAccounts,
  padInstructionData,
} from "./encoding.js";
import { trigger } from "./trigger.js";

describe("encodeTrigger", () => {
  it("packs a cron expression at the front of a 128-byte buffer", () => {
    const { triggerType, triggerData } = encodeTrigger(trigger.cron("0 * * * *"));
    expect(triggerType).toBe(0);
    expect(triggerData.length).toBe(TRIGGER_DATA_LEN);
    expect(new TextDecoder().decode(triggerData.subarray(0, 9))).toBe("0 * * * *");
  });

  it("rejects cron expressions over 40 bytes", () => {
    const long = "0 0 1 1 * 0 0 1 1 * 0 0 1 1 * 0 0 1 1 * 0 0";
    expect(() => encodeTrigger({ type: "cron", expression: long, timezoneOffsetSeconds: 0 })).toThrow();
  });

  it("writes the price-threshold direction tag at byte 40", () => {
    const feed = PublicKey.default;
    const { triggerType, triggerData } = encodeTrigger(
      trigger.priceThreshold(feed, 200_000_000n, "below", 50, 30),
    );
    expect(triggerType).toBe(2);
    expect(triggerData[40]).toBe(1);
  });

  it("tags slot vs epoch granularity at byte 0", () => {
    expect(encodeTrigger(trigger.slotPeriod(100)).triggerData[0]).toBe(0);
    expect(encodeTrigger(trigger.epoch()).triggerData[0]).toBe(1);
  });
});

describe("padInstructionData", () => {
  it("prefixes a little-endian u16 length", () => {
    const padded = padInstructionData(new Uint8Array([9, 8, 7]));
    expect(padded.length).toBe(TARGET_IX_DATA_LEN);
    expect(padded[0]).toBe(3);
    expect(padded[1]).toBe(0);
    expect(Array.from(padded.subarray(2, 5))).toEqual([9, 8, 7]);
  });

  it("rejects instruction data larger than the buffer", () => {
    expect(() => padInstructionData(new Uint8Array(TARGET_IX_DATA_LEN))).toThrow();
  });
});

describe("hashTargetAccounts", () => {
  it("is deterministic and order sensitive", () => {
    const a = { pubkey: new PublicKey("11111111111111111111111111111111"), isSigner: false, isWritable: true };
    const b = { pubkey: PublicKey.default, isSigner: true, isWritable: false };
    expect(hashTargetAccounts([a, b])).toEqual(hashTargetAccounts([a, b]));
    expect(hashTargetAccounts([a, b])).not.toEqual(hashTargetAccounts([b, a]));
  });
});

describe("encodeRegisterJobData", () => {
  it("produces the exact IDL byte length with the discriminator up front", () => {
    const { triggerType, triggerData } = encodeTrigger(trigger.cron("*/5 * * * *"));
    const data = encodeRegisterJobData({
      triggerType,
      triggerData,
      targetProgram: PublicKey.default,
      targetIxData: padInstructionData(new Uint8Array([1])),
      targetAccountsHash: new Uint8Array(32),
      budgetLamports: 50_000_000n,
      maxPriorityFeeMicroLamports: 5_000n,
    });
    expect(data.length).toBe(8 + 1 + 128 + 32 + 512 + 32 + 8 + 8);
    expect(Array.from(data.subarray(0, 8))).toEqual([193, 91, 44, 51, 30, 133, 240, 168]);
  });
});
