import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import type { Connection, Signer } from "@solana/web3.js";
import { trigger } from "./trigger.js";
import type { JobSpec, ScheduleResult } from "./types.js";
import { configPda, jobPda, keeperBondPda, registryPda } from "./pda.js";
import {
  encodeRegisterJobData,
  encodeTrigger,
  hashTargetAccounts,
  padInstructionData,
} from "./encoding.js";

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
   * Read the next global job index from the on-chain registry.
   *
   * The registry account stores `total_jobs` as a little-endian u64 at offset 8
   * (after the 8-byte Anchor discriminator). The program derives each Job PDA
   * from `(owner, job_index)`, so the client reads the counter before deriving.
   */
  async nextJobIndex(): Promise<bigint> {
    const info = await this.connection.getAccountInfo(this.registryPda());
    if (!info) {
      throw new Error(
        `registry account ${this.registryPda().toBase58()} not found on this ` +
          "cluster -- the VIGYL program is not initialized here yet",
      );
    }
    return info.data.readBigUInt64LE(8);
  }

  /**
   * Build the `register_job` instruction for a spec at a known job index.
   *
   * Pure and synchronous: encodes the trigger, pads the target instruction data,
   * hashes the account list, and assembles the account metas in IDL order.
   */
  buildRegisterJobIx(owner: PublicKey, jobIndex: bigint, spec: JobSpec): TransactionInstruction {
    const { triggerType, triggerData } = encodeTrigger(spec.trigger);
    const targetIxData = padInstructionData(spec.target.ixData);
    const targetAccountsHash = hashTargetAccounts(spec.target.accounts);
    const data = encodeRegisterJobData({
      triggerType,
      triggerData,
      targetProgram: spec.target.program,
      targetIxData,
      targetAccountsHash,
      budgetLamports: spec.budgetLamports,
      maxPriorityFeeMicroLamports: spec.maxPriorityFeeMicroLamports,
    });
    return new TransactionInstruction({
      programId: this.programId,
      keys: [
        { pubkey: owner, isSigner: true, isWritable: true },
        { pubkey: this.configPda(), isSigner: false, isWritable: false },
        { pubkey: this.registryPda(), isSigner: false, isWritable: true },
        { pubkey: this.jobPda(owner, jobIndex), isSigner: false, isWritable: true },
      ],
      data: Buffer.from(data),
    });
  }

  /**
   * Register a job on-chain and wait for confirmation.
   *
   * Reads the next job index from the registry, builds the `register_job`
   * instruction, signs with the wallet, and submits. The call reaches the live
   * program on whichever cluster `connection` points at; against a cluster where
   * the program is not initialized, `nextJobIndex()` throws before any fee is paid.
   */
  async schedule(spec: JobSpec): Promise<ScheduleResult> {
    const owner = this.wallet.publicKey;
    const jobIndex = await this.nextJobIndex();
    const ix = this.buildRegisterJobIx(owner, jobIndex, spec);
    const tx = new Transaction().add(ix);
    const signature = await sendAndConfirmTransaction(this.connection, tx, [this.wallet], {
      commitment: "confirmed",
    });
    return {
      jobPubkey: this.jobPda(owner, jobIndex),
      signature,
      jobIndex,
    };
  }
}
