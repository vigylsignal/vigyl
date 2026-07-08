import { Vigyl } from "@vigyl/sdk";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { readFileSync } from "node:fs";

// Schedules a `harvest()` CPI on your program every hour.
async function main() {
  const rpcUrl = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
  const programId = new PublicKey(
    process.env.VIGYL_PROGRAM_ID ?? "ErbqsQTo28e4vxCb8Jfzj4UERNrhFmJD7XmN4vERkQVK",
  );
  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(process.env.KEYPAIR!, "utf8"))),
  );

  const vigyl = new Vigyl({
    connection: new Connection(rpcUrl, "confirmed"),
    programId,
    wallet,
  });

  const harvestProgram = new PublicKey(process.env.HARVEST_PROGRAM!);
  const vaultAccount = new PublicKey(process.env.VAULT!);

  const preview = vigyl.previewSchedule(wallet.publicKey, 0n, {
    trigger: Vigyl.trigger.cron("0 * * * *"),
    target: {
      program: harvestProgram,
      ixData: new Uint8Array([13, 37]),
      accounts: [
        { pubkey: vaultAccount, isSigner: false, isWritable: true },
      ],
    },
    budgetLamports: 50_000_000n,
    maxPriorityFeeMicroLamports: 5_000n,
  });

  console.log("job pubkey:", preview.jobPubkey.toBase58());
  console.log("trigger type:", preview.triggerType);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
