# Scheduled Run Receipts

Scheduled Run Receipts (`srr`) is a local-first absence detector for cron jobs,
queues, and scheduled workflows. It turns an expected UTC cron calendar and
signed start/finish receipts into an auditable weekly HTML evidence page. A
missing run is evidence too: `srr check` exits non-zero after its grace window.

It is for small-team operators who need to answer two questions quickly:

- Did the scheduled work happen?
- If it did not happen, did our monitor notice?

`srr` does not schedule jobs, host queues, send alerts, or upload metadata. The
state file, job keys, receipts, and generated reports stay on your machine.

## Install

Build the single binary with stable Rust:

```sh
cargo install --path .
srr --help
```

The factory publishes release binaries later. The crate starts at `0.1.0` and
is ready to package with `cargo package`.

## Usage

Create an isolated monitor and register the expected schedule. Five-field cron
expressions are interpreted in UTC. `--data` may also be supplied through the
`SRR_DATA` environment variable.

```sh
export SRR_DATA="$PWD/.receipts/state.json"
srr init
srr job add database-backup --schedule "0 2 * * *" --grace 15m
```

For a job running on the same host, wrap it with local start/finish receipts:

```sh
run_id="backup-$(date -u +%Y%m%dT%H%M%SZ)"
srr run start database-backup --run-id "$run_id"
if ./backup.sh; then
  srr run finish database-backup --run-id "$run_id" --status success
else
  srr run finish database-backup --run-id "$run_id" --status failure
  exit 1
fi
```

For a remote runner, copy that job's key once, then issue portable receipts on
the runner and accept them on the monitor. Tokens are HMAC-SHA256 signed, bind
the job, expected slot, event, status, timestamp, run ID, and random nonce, and
are safe to transport in a POST body or message queue.

```sh
# On the monitor: export only this job's key for secure provisioning.
srr job key database-backup --output ./database-backup.key

# On the runner:
token=$(srr receipt sign start --job database-backup \
  --run-id backup-20260828T020000Z --key-file ./database-backup.key)

# On the monitor:
srr receipt accept "$token"
```

Accepted nonces are retained for seven days by default. Replaying the same
token fails. Receipts older than the retention window fail. Keep key files out
of source control and rotate a compromised key with `srr job rotate-key`.

Check the expected calendar in a script or CI. Exit code `0` means every due
slot is healthy; `2` means a missing, late, failed, running, or overlapping run
was found; input and state errors use exit code `1`.

```sh
srr check --since 7d
srr check --since 30d --json > status.json
srr export --week 2026-08-24 --output receipts-2026-W35.html
```

Schedule `srr check` independently at an interval shorter than your smallest
grace window, and route exit code `2` through the notification mechanism you
already trust. Detection is deterministic, but the CLI deliberately does not
run a daemon or choose an alert provider for you.

All reporting commands support `--json` where structured output is useful:

```sh
srr job list --json
srr status --json
srr check --json
```

`srr export` writes a standalone, script-free HTML file containing the weekly
expected-run calendar, receipt details, anomaly summary, and a SHA-256 evidence
digest. It can be opened offline or attached to an incident record.

## CLI reference

```text
srr init [--force]
srr job add <JOB> --schedule <CRON> [--grace <DURATION>]
srr job list [--json]
srr job key <JOB> --output <PATH> [--force]
srr job rotate-key <JOB>
srr run start <JOB> --run-id <ID> [--scheduled-at <RFC3339>]
srr run finish <JOB> --run-id <ID> --status <success|failure>
srr receipt sign <start|finish> --job <JOB> --run-id <ID> --key-file <PATH>
                 [--scheduled-at <RFC3339>] [--status <success|failure>]
srr receipt accept <TOKEN>
srr check [--since <DURATION>] [--json]
srr status [--json]
srr export [--week <YYYY-MM-DD>] --output <PATH>
```

Durations accept `s`, `m`, `h`, or `d` (for example `90s`, `15m`, `24h`,
`7d`). Job names may contain lowercase letters, digits, `_`, and `-`.

## Develop and verify

Prerequisites: stable Rust, Node.js 20+, and npm.

```sh
npm install
npm test              # Rust tests, site unit checks, and production build
npm run build         # exact factory build command; outputs dist/site/index.html
npm run build:site    # site only; outputs dist/site
cargo test
cargo package
```

Start the documentation site with `npm run dev`. Production deploys serve
`dist/site` at <https://scheduled-run-receipts.sociobot.in>.

## Data and security model

- All state is a versioned JSON file, written atomically with owner-only
  permissions on Unix.
- Each job receives an independent 256-bit secret. The secret is never present
  in JSON output or HTML exports.
- Receipt signatures are checked in constant time. Duplicate nonces and stale
  receipts are rejected before insertion. The nonce index expires after seven
  days; accepted local receipt history remains available for evidence exports.
- `srr` has no network client, telemetry, analytics, or background process.
- The static site has no cookies, tracking, remote fonts, or runtime CDN calls.

## License

MIT. See [LICENSE](LICENSE). Changes are recorded in [CHANGELOG.md](CHANGELOG.md).
