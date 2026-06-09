import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

const enc = (s: string) => new TextEncoder().encode(s);

export function configPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([enc("config")], programId);
}

export function registryPda(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([enc("registry")], programId);
}

export function jobPda(
  programId: PublicKey,
  owner: PublicKey,
  jobIndex: bigint | number,
): [PublicKey, number] {
  const idx = new BN(jobIndex.toString()).toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [enc("job"), owner.toBuffer(), idx],
    programId,
  );
}

export function keeperBondPda(programId: PublicKey, keeper: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [enc("keeper"), keeper.toBuffer()],
    programId,
  );
}

export function executionProofPda(
  programId: PublicKey,
  job: PublicKey,
  executionIndex: bigint | number,
): [PublicKey, number] {
  const idx = new BN(executionIndex.toString()).toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [enc("proof"), job.toBuffer(), idx],
    programId,
  );
}
