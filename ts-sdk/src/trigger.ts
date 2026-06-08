import { PublicKey } from "@solana/web3.js";

export type PriceDirection = "above" | "below";

export interface CronTrigger {
  type: "cron";
  expression: string;
  timezoneOffsetSeconds: number;
}

export interface AccountStateTrigger {
  type: "account_state";
  watchedAccount: PublicKey;
  expectedHashHex: string;
  dataOffset: number;
  dataLen: number;
}

export interface PriceThresholdTrigger {
  type: "price_threshold";
  pythFeed: PublicKey;
  thresholdPriceE6: bigint;
  direction: PriceDirection;
  maxConfidencePct: number;
  minPublishTimeSeconds: number;
}

export interface SlotEpochTrigger {
  type: "slot_epoch";
  granularity: "slot" | "epoch";
  periodSlots: number;
}

export type Trigger =
  | CronTrigger
  | AccountStateTrigger
  | PriceThresholdTrigger
  | SlotEpochTrigger;

export const trigger = {
  cron(expression: string, timezoneOffsetSeconds = 0): CronTrigger {
    if (expression.length > 40) {
      throw new Error("cron expression exceeds 40 chars");
    }
    return { type: "cron", expression, timezoneOffsetSeconds };
  },
  accountState(
    watchedAccount: PublicKey,
    expectedHashHex: string,
    dataOffset = 0,
    dataLen = 0,
  ): AccountStateTrigger {
    if (expectedHashHex.length !== 64) {
      throw new Error("expectedHashHex must be 32-byte hex");
    }
    return { type: "account_state", watchedAccount, expectedHashHex, dataOffset, dataLen };
  },
  priceThreshold(
    pythFeed: PublicKey,
    thresholdPriceE6: bigint,
    direction: PriceDirection = "above",
    maxConfidencePct = 100,
    minPublishTimeSeconds = 60,
  ): PriceThresholdTrigger {
    return {
      type: "price_threshold",
      pythFeed,
      thresholdPriceE6,
      direction,
      maxConfidencePct,
      minPublishTimeSeconds,
    };
  },
  slotPeriod(periodSlots: number): SlotEpochTrigger {
    if (periodSlots <= 0) throw new Error("periodSlots must be positive");
    return { type: "slot_epoch", granularity: "slot", periodSlots };
  },
  epoch(): SlotEpochTrigger {
    return { type: "slot_epoch", granularity: "epoch", periodSlots: 0 };
  },
};
