import type { PublicKey } from "@solana/web3.js";
import type { Trigger } from "./trigger.js";

export interface TargetInstruction {
  program: PublicKey;
  ixData: Uint8Array;
  accounts: {
    pubkey: PublicKey;
    isSigner: boolean;
    isWritable: boolean;
  }[];
}

export interface JobSpec {
  trigger: Trigger;
  target: TargetInstruction;
  budgetLamports: bigint;
  maxPriorityFeeMicroLamports: bigint;
}

export interface ScheduleResult {
  jobPubkey: PublicKey;
  signature: string;
  jobIndex: bigint;
}
