# @vigyl/sdk

TypeScript client for the VIGYL keeper network.

```bash
npm install @vigyl/sdk @solana/web3.js @coral-xyz/anchor
```

```typescript
import { Vigyl } from "@vigyl/sdk";

const vigyl = new Vigyl({
  connection,
  programId: VIGYL_PROGRAM_ID,
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
