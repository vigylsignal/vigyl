# vigyl-cli

Command-line interface for the VIGYL keeper network.

```bash
npm install -g vigyl-cli
vigyl --help
```

## Commands

- `vigyl job create` -- register a scheduled job
- `vigyl job list` -- list your jobs
- `vigyl job pause / resume / fund / cancel`
- `vigyl simulate` -- estimate execution cost + firing frequency
- `vigyl keeper bond / unbond / withdraw`
- `vigyl keeper run` -- start the bonded keeper daemon
- `vigyl leaderboard` -- top keepers by execution count

See [../docs/keeper-spec.md](../docs/keeper-spec.md) for the daemon boot sequence.
