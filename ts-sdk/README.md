# @vigyl/sdk

TypeScript client for the VIGYL keeper network.

Not on the npm registry yet -- build from the repo and consume via `npm link`
or a `file:` dependency:

```bash
npm ci && npm run build
```

```typescript
import { Vigyl } from "@vigyl/sdk";
import { Connection, PublicKey } from "@solana/web3.js";

// devnet deployment
const vigyl = new Vigyl({
  connection: new Connection("https://api.devnet.solana.com"),
  programId: new PublicKey("HH7mrDz4EUmPaZy8knZxB1SaPL6pvMiZm219YW99WU9o"),
  wallet: myKeypair,
});

const { jobPubkey } = await vigyl.schedule({
  trigger: Vigyl.trigger.cron("0 * * * *"),
  target: {
    program: myProgram,
    ixData: myInstructionData,
    accounts: myAccounts,
  },
  budgetLamports: 50_000_000n,
  maxPriorityFeeMicroLamports: 5_000n,
});
```

See [../docs/job-spec.md](../docs/job-spec.md) for the trigger encoding and
[../docs/architecture.md](../docs/architecture.md) for how the SDK, Anchor
program, and keeper daemon fit together.
