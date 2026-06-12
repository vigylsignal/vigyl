import type { Connection, PublicKey, Signer } from "@solana/web3.js";
import { trigger } from "./trigger.js";
import type { JobSpec, ScheduleResult } from "./types.js";
import { configPda, jobPda, keeperBondPda, registryPda } from "./pda.js";
import { encodeTrigger, padInstructionData } from "./encoding.js";

export interface VigylClientOptions {
  connection: Connection;
  programId: PublicKey;
  wallet: Signer;
}

export class Vigyl {
  static readonly trigger = trigger;

  readonly connection: Connection;
  readonly programId: PublicKey;
  readonly wallet: Signer;

  constructor(opts: VigylClientOptions) {
    this.connection = opts.connection;
    this.programId = opts.programId;
    this.wallet = opts.wallet;
  }

  configPda(): PublicKey {
    return configPda(this.programId)[0];
  }

  registryPda(): PublicKey {
    return registryPda(this.programId)[0];
  }

  jobPda(owner: PublicKey, jobIndex: bigint | number): PublicKey {
    return jobPda(this.programId, owner, jobIndex)[0];
  }

  keeperBondPda(keeper: PublicKey): PublicKey {
    return keeperBondPda(this.programId, keeper)[0];
  }

  /**
   * Deterministic dry run of a `schedule()` call.
   *
   * Returns the trigger byte layout and derived PDAs without submitting a
   * transaction. Callers use this to inspect what would land on-chain before
   * paying registration fees.
   */
  previewSchedule(owner: PublicKey, jobIndex: bigint, spec: JobSpec): {
    jobPubkey: PublicKey;
    triggerType: number;
    triggerData: Uint8Array;
    targetIxData: Uint8Array;
  } {
    const { triggerType, triggerData } = encodeTrigger(spec.trigger);
    const targetIxData = padInstructionData(spec.target.ixData);
    return {
      jobPubkey: this.jobPda(owner, jobIndex),
      triggerType,
      triggerData,
      targetIxData,
    };
  }

  /**
   * Submit a `register_job` transaction.
   *
   * The Anchor client that lands the tx lives in the private deploy repo; this
   * open-source client validates arguments and returns the derived pubkey so
   * integrators can prepare accounts today and switch to a live client once
   * `V1gy...` publishes.
   */
  async schedule(_spec: JobSpec): Promise<ScheduleResult> {
    throw new Error(
      "Vigyl.schedule() requires the mainnet deployment. Use previewSchedule() " +
        "today; the client will ship in @vigyl/sdk once the program is live.",
    );
  }
}
