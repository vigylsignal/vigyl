import { Vigyl } from "@vigyl/sdk";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

// Rebalance a vault whenever a specific 8-byte accumulator slot changes.
async function main() {
  const rpcUrl = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
  const programId = new PublicKey(process.env.VIGYL_PROGRAM_ID!);
  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(process.env.KEYPAIR!, "utf8"))),
  );

  const vigyl = new Vigyl({
    connection: new Connection(rpcUrl, "confirmed"),
    programId,
    wallet,
  });

  const vaultProgram = new PublicKey(process.env.VAULT_PROGRAM!);
  const watched = new PublicKey(process.env.WATCHED_ACCOUNT!);

  const accountInfo = await vigyl.connection.getAccountInfo(watched);
  if (!accountInfo) throw new Error("watched account missing");
  const slice = accountInfo.data.subarray(32, 40);
  const expectedHashHex = createHash("sha256").update(slice).digest("hex");

  const preview = vigyl.previewSchedule(wallet.publicKey, 1n, {
    trigger: Vigyl.trigger.accountState(watched, expectedHashHex, 32, 8),
    target: {
      program: vaultProgram,
      ixData: new Uint8Array([0x22]),
      accounts: [
        { pubkey: watched, isSigner: false, isWritable: true },
      ],
    },
    budgetLamports: 100_000_000n,
    maxPriorityFeeMicroLamports: 8_000n,
  });

  console.log("job pubkey:", preview.jobPubkey.toBase58());
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
