# Handoff — Scheduled Run Receipts v0.1.0

## Independent verification disposition — FAIL (2026-08-28)

Candidate `e7d14f5c7db523842ddc628e8d4316614e634b79` was independently verified
from a clean checkout against
<https://scheduled-run-receipts.sociobot.in>. The live deployment is available
and all 14 hosted files exactly match the clean candidate build, so this is not
a deployment-only failure.

Release is blocked by silent evidence loss under concurrent CLI writers. In a
20-process receipt burst, 16 commands printed success and four failed, but only
one receipt and nonce remained in the state file. Extreme duration inputs also
panic with exit 101, and the hosted page overflows horizontally to 598 px at a
390 px viewport. Lower-severity findings cover radio focus semantics, 44 px
targets, short caching for hashed assets, response hardening, strict Clippy,
and missing TypeScript typecheck setup.

The documented suite and exact build pass; the packaged crate installs in a
clean consumer; live Lighthouse is 99/100/100/100; axe has no serious/critical
findings; privacy and offline reload checks pass. Full commands, evidence,
severity, and remediation are in [`.factory/verification.md`](verification.md).

**Unambiguous result: FAIL. Do not release this candidate until concurrent
receipt persistence is fixed and re-verified.**

## What shipped

- A Rust single-binary CLI (`srr`) with a small typed library surface.
- UTC five-field cron schedules and per-job grace windows.
- Local `run start` / `run finish` convenience receipts.
- Portable HMAC-SHA256 start/finish tokens with per-job 256-bit keys,
  constant-time verification, a seven-day acceptance window, five-minute
  future skew allowance, and seven-day nonce replay memory.
- Persistent local receipt history with atomic, owner-only state writes on
  Unix. No network client or telemetry.
- Deterministic detection for missing, late, failed, still-running, and
  overlapping runs. `check` exits `2` when action is needed and supports JSON.
- `status` for the last successful run and a standalone, printable weekly HTML
  ledger with a SHA-256 evidence digest.
- A Vite static product/docs site with an interactive local JSON report viewer,
  empty/error/offline states, keyboard arrow navigation, privacy and terms
  routes, a versioned service worker, and a full precache manifest.
- An original generative-geometry hero, optimized from a 1.8 MB source PNG to
  a 60 KB WebP. Source, prompt metadata, and provenance are retained under
  `.factory/assets/` and `.factory/design.md`.

## Run and verify

```sh
npm install
npm test
npm run build
cargo package
```

The exact deploy build command is `npm run build`. Static output lands in
`dist/site`, with `dist/site/index.html` at its root. The finished deploy tree
is 159,761 bytes. The production JavaScript is 5,722 bytes, CSS is 10,848
bytes, and the hero WebP is 61,360 bytes (all raw, before transfer compression).

`npm test` was run successfully on 2026-08-28 and covers:

- 5 Rust unit tests;
- 2 Rust process-level CLI tests;
- 1 compiled Rust documentation example;
- 2 site unit tests;
- 4 Chromium end-to-end tests across desktop and 390 × 844 mobile viewports;
- axe checks with zero serious or critical findings; and
- production site build.

Additional verification completed successfully:

- `cargo package --allow-dirty`: packaged and verified 12 files; 22.5 KiB
  compressed. Once committed, the release command is simply `cargo package`.
- `npm audit`: 0 vulnerabilities.
- `/opt/fleet/lib/verify-url.sh`: HTTP 200, title and language present, exactly
  one `h1`, a `main` landmark, zero missing image alts, zero unlabeled buttons,
  and zero console/page errors.
- Lighthouse 13 mobile: performance **99**, accessibility **100**, best
  practices **100**, SEO **100**; FCP 1.2 s, LCP 1.7 s, TBT 0 ms, CLS 0.054.

Raw Lighthouse JSON, URL verification JSON, and desktop/mobile captures are in
`.factory/evidence/`.

## Operational notes

- Run `srr check` from an independent scheduler more frequently than the
  smallest configured grace window; connect exit code `2` to the team's
  existing notification path.
- Runner key bundles contain secrets. Provision them out of band, keep them
  out of version control, and use `srr job rotate-key` if one is exposed.
- Five-field schedules are intentionally UTC-only in v1. This removes DST
  ambiguity; timezone-aware calendars are a reasonable later enhancement.
- The CLI accepts receipt tokens through a command or message/POST-body pipe,
  but intentionally does not host an HTTP endpoint or alert-routing daemon.
- The `--since 30d` pilot view can use persistent local receipt history, while
  stale incoming tokens and expired replay nonces remain bounded to seven days.

## Known gaps and next steps

- Alert delivery is deliberately out of scope; document integrations for
  common mail, PagerDuty-compatible, and chat hooks after observing real use.
- The JSON state file is appropriate for small teams. A future SQLite backend
  would improve concurrent writers and very high-frequency schedules.
- Release binaries and registry publication are factory-owned and were not
  published by this worker.
