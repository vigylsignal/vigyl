#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { Command } from "commander";
import chalk from "chalk";

const API_BASE = process.env.VIGYL_API ?? "https://vigyl.cloud/api";

const orange = chalk.hex("#FF7A29");
const amber = chalk.hex("#F5A623");
const grey = chalk.hex("#C7CCD6");

// docs/job-spec.md trigger discriminants
const TRIGGER_TYPES: Record<string, number> = {
  cron: 0,
  account_state: 1,
  price_threshold: 2,
  slot_epoch: 3,
};

async function getJson(path: string): Promise<any> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`GET ${API_BASE}${path} -> HTTP ${res.status}`);
  return res.json();
}

async function postJson(path: string, body: unknown): Promise<any> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new Error(`POST ${API_BASE}${path} -> HTTP ${res.status}: ${await res.text()}`);
  }
  return res.json();
}

function fail(err: unknown): never {
  console.error(chalk.red(err instanceof Error ? err.message : String(err)));
  process.exit(1);
}

const program = new Command();

program
  .name("vigyl")
  .description("VIGYL keeper network command line")
  .version("0.1.0");

const job = program.command("job").description("manage automation jobs");

job
  .command("create")
  .description("register a new job")
  .option("--cron <expr>", "cron expression, e.g. \"0 * * * *\"")
  .option("--account-state <pubkey>", "account to watch")
  .option("--price-feed <pubkey>", "Pyth price feed pubkey")
  .option("--threshold <price_e6>", "price threshold * 1e6")
  .option("--direction <above|below>", "threshold direction", "above")
  .option("--slot-period <n>", "period in slots", (v) => parseInt(v, 10))
  .option("--target <program>", "target program pubkey")
  .option("--ix-file <path>", "instruction data JSON")
  .option("--budget <sol>", "initial budget in SOL")
  .option("--max-fee <micro>", "priority fee cap (micro-lamports/CU)", (v) => parseInt(v, 10))
  .action(() => {
    console.log(chalk.hex("#FF7A29")("job create -- pending mainnet deploy"));
  });

job
  .command("list")
  .description("list registered jobs from the indexer")
  .option("--owner <pubkey>", "filter by job owner")
  .option("--status <state>", "active or paused")
  .action(async (opts: { owner?: string; status?: string }) => {
    try {
      const params = new URLSearchParams();
      if (opts.owner) params.set("owner", opts.owner);
      if (opts.status) params.set("status", opts.status);
      const query = params.toString();
      const data = await getJson(`/jobs${query ? `?${query}` : ""}`);
      if (data.jobs.length === 0) {
        console.log(grey(`no jobs indexed (total ${data.total})`));
        return;
      }
      for (const j of data.jobs) {
        console.log(
          `${orange(j.pubkey)}  ${amber(j.trigger_type_name)}  ${j.status}  ` +
            `execs ${j.execution_count}  budget ${j.budget_lamports} lamports`,
        );
      }
    } catch (err) {
      fail(err);
    }
  });

for (const name of ["pause", "resume", "fund", "cancel"]) {
  job
    .command(`${name} <job>`)
    .action(() => console.log(chalk.hex("#C7CCD6")(`${name} -- pending mainnet deploy`)));
}

program
  .command("simulate <spec>")
  .description("estimate execution cost for a job spec json (docs/job-spec.md)")
  .option("--cu <units>", "estimated compute units", (v) => parseInt(v, 10), 200_000)
  .action(async (specPath: string, opts: { cu: number }) => {
    try {
      const spec = JSON.parse(readFileSync(specPath, "utf8"));
      const triggerType = TRIGGER_TYPES[spec.trigger?.type];
      if (triggerType === undefined) {
        throw new Error(`unknown trigger type: ${JSON.stringify(spec.trigger?.type)}`);
      }
      const body: Record<string, unknown> = {
        trigger_type: triggerType,
        estimated_compute_units: opts.cu,
        cron_expression: spec.trigger?.expression ?? null,
        period_slots: spec.trigger?.period_slots ?? null,
        target_program: spec.target?.program ?? null,
      };
      const maxFee = spec.budget?.max_priority_fee_micro_lamports;
      if (maxFee !== undefined) body.max_priority_fee_micro_lamports = maxFee;
      const quote = await postJson("/quote", body);
      console.log(orange(`cost per execution: ${quote.cost_per_execution_lamports} lamports`));
      console.log(amber(`expected executions/day: ${quote.expected_executions_per_day}`));
      console.log(
        amber(
          `priority fee applied: ${quote.priority_fee_applied} micro-lamports/cu ` +
            `(${quote.fee_source})`,
        ),
      );
      console.log(grey(`estimated daily cost: ${quote.estimated_daily_cost_sol} SOL`));
      console.log(grey(`estimated monthly cost: ${quote.estimated_monthly_cost_sol} SOL`));
      for (const warning of quote.warnings ?? []) {
        console.log(chalk.yellow(`warning: ${warning}`));
      }
    } catch (err) {
      fail(err);
    }
  });

const keeper = program.command("keeper").description("bonded keeper commands");

keeper
  .command("bond <amount>")
  .action(() => console.log(chalk.hex("#FF7A29")("bond -- pending mainnet deploy")));

keeper
  .command("unbond")
  .action(() => console.log(chalk.hex("#F5A623")("unbond -- pending mainnet deploy")));

keeper
  .command("withdraw")
  .action(() => console.log(chalk.hex("#C7CCD6")("withdraw -- pending mainnet deploy")));

keeper
  .command("run")
  .description("start the bonded keeper daemon")
  .option("--min-fee <micro>", "minimum priority fee accepted", (v) => parseInt(v, 10))
  .option("--max-concurrent <n>", "max concurrent executions", (v) => parseInt(v, 10), 3)
  .option("--health-port <port>", "health check http port", (v) => parseInt(v, 10))
  .action(() => {
    console.log(chalk.hex("#FF7A29")("keeper run -- daemon entry point"));
  });

program
  .command("leaderboard")
  .description("top keepers by execution count")
  .option("--window <w>", "1d, 7d, 30d, or all", "7d")
  .action(async (opts: { window: string }) => {
    try {
      const data = await getJson(`/leaderboard?window=${opts.window}`);
      if (data.entries.length === 0) {
        console.log(grey(`no executions in the ${data.window} window yet`));
        return;
      }
      data.entries.forEach((e: any, i: number) => {
        console.log(
          `${amber(String(i + 1).padStart(3))}  ${orange(e.keeper_pubkey)}  ` +
            `execs ${e.executions}  success ${(e.success_rate * 100).toFixed(1)}%`,
        );
      });
    } catch (err) {
      fail(err);
    }
  });

program
  .command("stats")
  .description("network counters from the indexer")
  .action(async () => {
    try {
      const s = await getJson("/stats");
      console.log(orange(`jobs live: ${s.jobs_live} (total ${s.jobs_total})`));
      console.log(amber(`executions: ${s.executions_total}`));
      console.log(amber(`keepers bonded: ${s.keepers_bonded}`));
      console.log(grey(`slashes: ${s.slashes_total}`));
      console.log(grey(`indexer lag: ${s.indexer_lag_slots} slots`));
    } catch (err) {
      fail(err);
    }
  });

program.parseAsync(process.argv);
