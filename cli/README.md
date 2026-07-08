# vigyl-cli

Command-line interface for the VIGYL keeper network.

```bash
npm install -g vigyl-cli
vigyl --help
```

## Commands

- `vigyl job create` -- register a scheduled job
- `vigyl job list` -- list registered jobs from the indexer
- `vigyl job pause / resume / fund / cancel`
- `vigyl simulate <spec.json>` -- estimate execution cost + firing frequency for a [job spec](../docs/job-spec.md)
- `vigyl keeper bond / unbond / withdraw`
- `vigyl keeper run` -- start the bonded keeper daemon
- `vigyl leaderboard` -- top keepers by execution count
- `vigyl stats` -- network counters from the indexer

`job list`, `simulate`, `leaderboard`, and `stats` read the public indexer API
(`https://vigyl.cloud/api`, override with `VIGYL_API`).

See [../docs/keeper-spec.md](../docs/keeper-spec.md) for the daemon boot sequence.
