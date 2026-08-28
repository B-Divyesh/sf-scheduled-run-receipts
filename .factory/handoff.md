# Handoff — Scheduled Run Receipts v0.1.0

## Repair verification — pending deployment (2026-08-28)

This repair resolves every finding in the independent verification of candidate
`e7d14f5c7db523842ddc628e8d4316614e634b79`.

- Every mutation now holds a cross-process advisory lock across state load,
  mutation, atomic replacement, and durable sync. Temporary filenames are
  unique. A 20-process process-level regression asserts all 20 accepted
  receipts and all 20 replay nonces remain in state.
- Duration parsing uses Chrono's checked constructors, so both documented
  `i64::MAX` duration inputs return exit 1 and leave the store unchanged.
- The hero and document clip decorative horizontal overflow. Browser tests
  assert `scrollWidth === clientWidth` at desktop and 390 px.
- The evidence radio group has one roving Tab stop and supports Arrow/Home/End.
  Every visible link, button, and file label is regression-tested at 44 px or
  larger.
- `staticwebapp.config.json` supplies immutable caching for hashed assets,
  no-cache service-worker delivery, CSP, framing, permissions, referrer,
  nosniff, and HSTS headers. A themed 404 artifact is shipped through the
  static-host response override.
- Strict `cargo clippy -D warnings` and `npm run typecheck` are repository
  gates. The prior Clippy warnings are fixed.

Clean verification completed from this worktree:

```text
npm ci                                      PASS (0 vulnerabilities)
npm test                                    PASS
cargo fmt --all -- --check                  PASS
cargo clippy --all-targets --all-features -- -D warnings   PASS
npm run test:e2e                            PASS (10/10: desktop + 390 px)
```

The browser suite has zero console errors and zero axe serious/critical
violations. It proves service-worker offline reload and checks that local JSON
report loading makes only same-origin requests. Claim commands and sandbox
details are in [`.factory/claims.json`](claims.json); the disposable CLI demo
is documented in [`.factory/demo.md`](demo.md).

Packaging was also verified after the repair commit:

```text
cargo package --locked                              PASS (14 files, 25.0 KiB compressed)
CARGO_INSTALL_ROOT=<fresh temp> cargo install --locked --path target/package/scheduled-run-receipts-0.1.0   PASS
<fresh temp>/bin/srr --version                      srr 0.1.0
<fresh temp>/bin/srr demo                           PASS (state + weekly HTML paths printed)
```

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
- 5 Rust process-level CLI tests, including the exact 20-process persistence
  burst, checked duration bounds, and disposable sample ledger;
- 1 compiled Rust documentation example;
- 2 site unit tests;
- 4 Chromium end-to-end tests across desktop and 390 × 844 mobile viewports;
- 10 Chromium end-to-end tests across desktop and 390 × 844 mobile viewports,
  including offline reload, privacy, roving focus, 44 px targets, and overflow;
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
- The JSON state file is appropriate for small teams. Locking preserves all
  accepted concurrent writes; a future SQLite backend could improve throughput
  for very high-frequency schedules.
- Release binaries and registry publication are factory-owned and were not
  published by this worker.
