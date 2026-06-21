#!/usr/bin/env node
import { Command } from "commander";
import chalk from "chalk";

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
  .option("--owner <pubkey>")
  .action(() => {
    console.log(chalk.hex("#F5A623")("job list -- reads /jobs from vigyl.cloud"));
  });

for (const name of ["pause", "resume", "fund", "cancel"]) {
  job
    .command(`${name} <job>`)
    .action(() => console.log(chalk.hex("#C7CCD6")(`${name} -- pending mainnet deploy`)));
}

program
  .command("simulate <spec>")
  .description("simulate a job spec without registering")
  .action(() => console.log(chalk.hex("#C7CCD6")("simulate -- calls vigyl.cloud /quote")));

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
  .action(() => console.log(chalk.hex("#F5A623")("leaderboard -- reads /leaderboard")));

program.parseAsync(process.argv);
