# Independent product verification 2 — FAIL

**Implementation candidate:** `ff88b4e3606fd74388a2c54d98484a4c177bb946`

**Candidate documentation SHA:** `ff88b4e3606fd74388a2c54d98484a4c177bb946` (the repair handoff was updated in the same candidate commit)

**Prior verification documentation SHA:** `b0a9bf420fe5a10e677e17e9cbc5ff8758067805`

**Live URL:** <https://scheduled-run-receipts.sociobot.in>

**Verified:** 2026-09-05 UTC

## Verdict

**FAIL — do not release.** There are four findings, including one live release-blocking defect, and 18 identified public claims without the required claim-test coverage. A clean candidate test run passing does not make the live offline claim true.

## Findings

### High — the live site cannot work offline after its first visit

The public claim in `.factory/claims.json` says, “The site works offline after the first visit.” It passed only against Vite preview. It fails on the actual static host.

A fresh desktop and phone context both had no service-worker registration after 12 seconds. Registering the same `/sw.js` manually began installation, then ended with no active, waiting, or installing worker. Its precache calls `cache.addAll()` with `/staticwebapp.config.json`; that URL is deliberately handled as deployment configuration and returns HTTP 404 on the live host. `cache.addAll()` therefore rejects and the worker cannot activate. After a fresh first visit, switching the context offline and reloading produced `net::ERR_INTERNET_DISCONNECTED`, not the site shell.

The 404 itself is expected and is not this finding. Including that expected 404 in the service-worker precache is the defect. The browser test does not represent the deployment because Vite preview serves that file with 200.

### Medium — the browser sample is not the required resettable demo sandbox

On fresh desktop and phone pages, “Try it with sample data” scrolls to a useful seven-slot `database-backup` sample with two exceptions. Selecting the missing slot gives a realistic absence explanation, and browser storage remained empty. However, there is no `/demo` or `?demo=1` entry, no persistent “Demo — sample data, nothing is saved” label, no **Reset demo**, and no **Start for real** action. The only label is the viewer-local sentence “Demo evidence is shown.”

The CLI `srr demo` is correctly disposable and passed in a clean installed consumer environment. The landing-page sample still fails the demo-sandbox contract and cannot demonstrate resetting or leaving a separate browser demo namespace.

### Medium — the first screen and copy do not meet the plain-words contract

The first `<h1>` is “Prove the run. Notice the gap.” The sentence below does not name the small-team cron/queue/workflow operator it is for. “Install the CLI” is visually primary while “Try it with sample data” is secondary. This does not state the job, audience, and first action in the required form before scrolling.

The same mood/metaphor style continues in headings such as “A week, accounted for,” “Expectation → receipt → proof,” and “Put proof beside the job.” The required `.factory/copy-audit.md` is absent. This is a contract failure, not an accessibility failure.

### Medium — the claim registry is incomplete: 18 public claims are untested

Only `offline-reload` and `local-viewer-private` appear in `.factory/claims.json`. Both declared commands were run; they pass in the local Vite sandbox, though `offline-reload` is false live as described above. The required one-claim/one-tag coverage is absent for 18 other material public promises identified across the landing page, README, and privacy copy. They include receipt/calendar detection; five anomaly states; HMAC receipts and per-job key isolation; replay protection; local-only state; no daemon, telemetry, or upload; disposable CLI demo isolation; documented exit codes; standalone export; offline/open/print export behavior; SHA-256 evidence digest; owner-only state; absence of third-party website services; and deletion by removing local files.

Some have ordinary Rust tests, but none has the required `@claim:<id>` entry and observable clean-sandbox claim command. The manifest therefore cannot prove the claims visitors are asked to rely on.

### Low — required metadata is incomplete

All routes have titles and one `<h1>`, and Privacy, Terms, and the designed 404 render correctly. The landing title remains slogan-like rather than “name — plain job.” None of the four HTML routes supplies a canonical link, Open Graph/Twitter card metadata, or the required 180 px Apple touch icon. These are missing mandatory site-structure elements.

## Evidence and checks that passed

Verification used a clean detached checkout of the implementation candidate with Node 22.23.2, npm 10.9.8, rustc/cargo 1.98.0, and Playwright 1.58.2.

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 0 vulnerabilities |
| `npm test` | PASS — Rust tests, strict TypeScript, site unit tests, production build, and 10 Playwright tests |
| Both declared claim commands | PASS locally — 2 desktop/mobile cases each |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo package --locked` | PASS — 14 files, 25.0 KiB compressed |
| Clean consumer install and `srr demo` | PASS — installed `srr 0.1.0`, generated a temp state file and standalone weekly HTML |

The installed CLI was also exercised through a normal local start receipt, invalid duration (`1x` and `-1s`), both documented `i64::MAX` duration boundaries, corrupt-state error, and `init --force` recovery. Invalid input returned exit 1 without a panic; recovery recreated a mode-0600 valid store. The candidate's 20-process receipt writer regression passed. These establish that the earlier high and medium CLI findings are resolved.

Fresh live desktop (1440 × 1000) and phone (390 × 844) contexts had no console or page errors, zero axe serious/critical violations, no horizontal overflow, a single roving radio Tab stop, and no visible control below 44 px. Arrow/Home keyboard selection worked. The live page made requests only to its own origin; the local report viewer loaded an empty report with the visible local-only status. Privacy and Terms were HTTP 200 with correct route titles. An unknown URL returned the themed page with deliberate HTTP 404, which is correct.

The repaired response headers, immutable hashed-asset cache policy, and 404 override are live. All 14 browser-served artifacts hash-match the clean candidate build. `staticwebapp.config.json` is the one build-tree file not served by the host (HTTP 404), as expected for host configuration; that difference exposed the service-worker defect above. The repository source link returned HTTP 200.

## Earlier verification findings

The concurrent-write evidence loss, extreme-duration panic, horizontal mobile overflow, radio Tab-order issue, undersized targets, short hashed-asset cache, missing hardening headers, 200 unknown-route fallback, strict Clippy warnings, and absent TypeScript gate recorded in `b0a9bf4` are resolved in this candidate. The offline live-host failure, demo-sandbox gap, copy/first-screen gap, incomplete claim registry, and metadata gaps are new or were not covered by the previous verification and remain open.

## Scope notes

This is a static site plus local CLI. It has no product backend, tenant API, health endpoint, account system, or rate-limited request allowance, so backend tenant-isolation, restart, health, and 429/`Retry-After` checks are not applicable. Local cross-process persistence was tested instead.
