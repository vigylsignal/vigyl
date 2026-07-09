import { Vigyl } from "@vigyl/sdk";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { readFileSync } from "node:fs";

// Fire a liquidation helper when SOL/USD crosses $200 from below.
async function main() {
  const rpcUrl = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
  const programId = new PublicKey(
    process.env.VIGYL_PROGRAM_ID ?? "64RwVTiRAtkVcjFTNaLorFbqRy2ifmP3kWmjJqgszrrh",
  );
  const wallet = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(process.env.KEYPAIR!, "utf8"))),
  );

  const vigyl = new Vigyl({
    connection: new Connection(rpcUrl, "confirmed"),
    programId,
    wallet,
  });

  const solUsdFeed = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
  const liquidator = new PublicKey(process.env.LIQUIDATOR_PROGRAM!);

  const preview = vigyl.previewSchedule(wallet.publicKey, 2n, {
    trigger: Vigyl.trigger.priceThreshold(solUsdFeed, 200_000_000n, "above", 50, 30),
    target: {
      program: liquidator,
      ixData: new Uint8Array([0x11]),
      accounts: [
        { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
      ],
    },
    budgetLamports: 80_000_000n,
    maxPriorityFeeMicroLamports: 10_000n,
  });

  console.log("job pubkey:", preview.jobPubkey.toBase58());
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
