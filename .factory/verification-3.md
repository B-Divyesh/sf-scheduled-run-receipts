# Verify scheduled runs and detect missing runs — verification 3

## Verdict

**FAIL.** There are **5 findings**: 0 critical, 1 high, 3 medium, and
1 low. There are **18 untested public claims** that are absent from the claim
registry. A PASS requires zero findings and zero untested claims.

| Item | Value |
| --- | --- |
| Implementation candidate reviewed | `ff88b4e3606fd74388a2c54d98484a4c177bb946` |
| Documentation baseline | `b5eab89fbc9baf5de68383d363ad2ae14532fe7c` |
| Live URL | <https://scheduled-run-receipts.sociobot.in> |
| Verified | 2026-09-06 UTC |
| Findings | 5 |
| Untested public claims | 18 |

## Candidate and live identity

`ff88b4e` is the last implementation commit. `b5eab89` changes only
`.factory/handoff.md` and `.factory/verification-2.md`. The only remote branch
is `main` at `b5eab89`; there is no later tag or branch.

There is no current evidence that repair2 published a distinct candidate. A
clean build from `b5eab89` uses the implementation from `ff88b4e`. The live
root HTML and service worker match that clean build byte for byte:

- root SHA-256: `1eaaccbbc41ceecbbc2a566f90ba84be4790de811ddcc32178ef8898bce52134`
- service-worker SHA-256: `2173305ce962b88fa0d06e34297aa1bb3eae8b04173cfc75a42aa3affb717c3a`
- 15 browser-served build files match their local SHA-256 digests
- `staticwebapp.config.json` is the only non-served build file; its live HTTP
  404 is expected because Azure consumes it as configuration
- live `Last-Modified` is 2026-08-28 09:35:48 UTC, after the `ff88b4e`
  implementation and before the verification-only `b5eab89` commit

The missing repair2 worker evidence is recorded here. It is not treated as a
reason to rebuild working code or change the deployment.

## First screen before scrolling

Fresh desktop and phone contexts both started at scroll position 0.

- Job shown: “Prove the run. Notice the gap.” The next copy explains signed
  receipts and an expected UTC calendar, but the heading does not plainly name
  the job.
- Audience shown: none. The copy does not name small-team operators who run
  cron jobs, queues, or scheduled workflows.
- First action shown: the filled primary action is “Install the CLI.” “Try it
  with sample data” is the secondary action.

## Current findings

### F1 — High — The live offline claim is false

The registered claim says the site works offline after its first visit. Its
declared test passes against local Vite preview but fails on the live host in
fresh desktop and phone contexts.

The built and live service worker precaches `/staticwebapp.config.json`. Azure
correctly returns HTTP 404 for that deployment-only file. `cache.addAll()`
therefore rejects during installation. After three seconds, each fresh browser
had no controller and no registration. Offline reload then returned
`net::ERR_INTERNET_DISCONNECTED`. The declared live test waited 20 seconds for
`navigator.serviceWorker.ready` and timed out on both devices.

Action: exclude deployment-only files from the precache, deploy, and run the
declared offline test against the live host in fresh desktop and phone
contexts.

### F2 — Medium — The browser sample is not a demo sandbox

“Try it with sample data” opens a useful seven-slot `database-backup` sample in
one click. Selecting the missing slot shows the expected time, grace window,
and absence record. Local storage, session storage, IndexedDB, and cookies stay
empty, and all requests stay on the product origin.

The required persistent label “Demo — sample data, nothing is saved” is
absent. “Reset demo” and “Start for real” are absent. `/demo` returns the
designed HTTP 404. `/?demo=1` returns the normal landing page without a distinct
demo mode. The shorter viewer sentence “Demo evidence is shown” does not supply
the required controls.

Action: add `/demo` or a real `?demo=1` mode with the persistent label, reset,
and leave-demo actions. Keep its storage separate from real browser data.

### F3 — Medium — The first screen and site copy do not meet the plain-words contract

The first heading is “Prove the run. Notice the gap.” The audience is absent,
the sample action is secondary, and the required three short privacy, offline,
and price facts are absent. The site also uses non-literal headings such as “A
week, accounted for,” “Expectation → receipt → proof,” and “Put proof beside
the job.” The required `.factory/copy-audit.md` does not exist.

Action: name the scheduled-run verification job and the small-team operator in
the first screen, make the sample the primary action, add the three plain facts,
replace the non-literal headings, and add the required copy audit.

### F4 — Medium — Eighteen public claims lack claim-registry coverage

`.factory/claims.json` contains only `offline-reload` and
`local-viewer-private`. The following 18 material promises appear on the site,
in the README, or in legal copy without their own registry entry and tagged
clean-sandbox test:

1. expected calendars and receipts detect missed scheduled work;
2. the five named anomaly states are classified;
3. portable receipts are HMAC-SHA256 signed and bind the documented fields;
4. each job has an independent 256-bit key;
5. stale and replayed receipts are rejected with retained nonces;
6. CLI state, keys, receipts, and reports remain local and are not uploaded;
7. the CLI runs no daemon;
8. the CLI has no network client, telemetry, analytics, or background process;
9. `srr demo` is disposable and does not read or write `SRR_DATA` or `--data`;
10. exit codes 0, 1, and 2 have the documented meanings;
11. JSON output is available for the documented reporting commands;
12. weekly export is standalone and script-free;
13. an exported ledger opens offline;
14. an exported ledger prints cleanly;
15. an export includes a SHA-256 evidence digest;
16. state writes are atomic and owner-only on Unix;
17. the website uses no analytics, cookies, remote fonts, or third-party scripts;
18. local data can be deleted by removing the documented files.

Ordinary Rust tests and manual checks prove several behaviors, but they do not
meet the required one-entry, one-`@claim:<id>` contract.

Action: add one claim entry and one observable clean-sandbox tagged test for
each promise, or remove the promise. Keep quantitative and privacy assertions
in the tests.

### F5 — Low — Required route metadata and shared site structure are incomplete

All four HTML documents have a title, description, language, theme color, one
`h1`, and header/main/footer landmarks. None has a canonical link, Open Graph
metadata, Twitter card metadata, or an Apple touch icon. There is no 1200×630
social image. The landing title is slogan-like rather than “product name —
plain job.” Headers are not consistent and the landing header omits Privacy.
Footers omit “Built by Param Factory” and a version or build ID. The external
Source link does not identify itself as external.

Action: complete the required metadata and use the standard header and footer
content on every route.

## Declared claims

| Claim | Local declared command | Live result |
| --- | --- | --- |
| `offline-reload` | PASS, 2/2 desktop and phone | **FAIL**, 0/2; service worker never becomes ready |
| `local-viewer-private` | PASS, 2/2 desktop and phone | PASS; local file status is explicit and requests are same-origin only |

The local offline result does not override the failed live result because Vite
preview serves `staticwebapp.config.json` while Azure does not.

## Clean checkout and installed artifact

Verification used a fresh clone at `b5eab89`. The implementation under test is
`ff88b4e`. Toolchain: Node 22.23.2, npm 10.9.8, rustc 1.98.0, Cargo 1.98.0,
and Playwright 1.58.2.

| Check | Result |
| --- | --- |
| `npm ci` | PASS, 61 packages, 0 vulnerabilities |
| `npm test` | PASS: 5 Rust unit, 5 CLI, 1 doctest, 2 site unit, 10 browser tests |
| `npm run test:e2e -- --grep @claim:offline-reload` | PASS locally, 2/2 |
| `npm run test:e2e -- --grep @claim:local-viewer-private` | PASS locally, 2/2 |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `npm audit --audit-level=low` | PASS, 0 vulnerabilities |
| `cargo package --locked` | PASS, 14 files, 25.0 KiB compressed |
| Fresh `cargo install --locked --path target/package/...` | PASS, `srr 0.1.0` |

The installed artifact completed `srr demo` and produced one realistic job,
nine receipts, and an HTML report containing success plus missing, late,
failed, running, and overlap states. The state file was mode 0600. A fresh
normal init/add/start/finish/status flow passed. Invalid `1x` and
`9223372036854775807s` durations returned exit 1 without panic. Corrupt JSON
returned exit 1 with a parse error; `init --force` recovered a valid empty
mode-0600 store.

The automated 20-process writer test retained every accepted receipt. This
also verifies local persistence across separate CLI processes. The package has
no network-client dependency or network call in its CLI source.

## Live browser, accessibility, privacy, and performance

- Factory URL verification: HTTP 200, title and `lang` present, one `h1`, one
  `main`, no missing alt text, no unlabeled buttons, and no console errors.
- Fresh 1440px desktop and 390×844 phone contexts had no horizontal overflow,
  no console or page errors, one roving radio tab stop, and no visible target
  below 44px.
- Axe found zero serious or critical issues on the landing, Privacy, Terms,
  and designed 404 pages in both viewports.
- The first Tab reaches the skip link. Its focus treatment is a 3px light
  outline plus a 6px vermilion ring. Radio Home, End, arrows, and Space work.
- Invalid report JSON gives an actionable message. A valid empty report then
  recovers immediately and shows the empty state.
- Reduced motion changes smooth scroll to `auto` and transitions to 0.01ms.
  Nothing loops or flashes. A 200% text-size smoke check retained main content
  and produced no horizontal overflow in either viewport.
- Root, Privacy, and Terms return HTTP 200 with distinct titles. An unknown URL
  returns the designed page with deliberate HTTP 404. That 404 is expected and
  is not a defect. All shipped links resolve; the public source link returns
  HTTP 200.
- Initial and sample-flow requests use only
  `scheduled-run-receipts.sociobot.in`. Storage remained empty before and after
  the sample flow.
- Security headers include CSP with response-level `frame-ancestors`, HSTS,
  nosniff, referrer policy, permissions policy, and frame denial. Hashed assets
  use a one-year immutable cache policy.
- Fresh Lighthouse 13 mobile: performance 99, accessibility 100, best
  practices 100, SEO 100; FCP 1.1s, LCP 1.4s, TBT 0ms, CLS 0.053, 113 KiB
  transferred.
- The clean build contains 5,843 bytes of JavaScript, 11,001 bytes of CSS,
  69,280 bytes of fonts, and a 61,360-byte hero image. These pass the budgets.

## Earlier findings

| Earlier finding | Current disposition |
| --- | --- |
| Concurrent writers lost accepted receipts | Resolved; 20-process regression passes |
| Extreme validly parsed durations panicked | Resolved; both limits return exit 1 |
| Desktop and mobile horizontal overflow | Resolved in both live viewports |
| Seven radios were all tabbable | Resolved; one roving tab stop |
| Visible controls were below 44px | Resolved; live regression passes |
| Hashed assets had short cache lifetime | Resolved; one-year immutable headers |
| CSP, framing, permissions, and HSTS were incomplete | Resolved |
| Unknown routes returned landing HTTP 200 | Resolved; designed HTTP 404 |
| Strict Clippy failed | Resolved |
| TypeScript gate was missing | Resolved and passing |
| Live offline claim failed | Open as F1 |
| Browser demo controls and label were missing | Open as F2 |
| First screen and copy failed plain-words requirements | Open as F3 |
| 18 public claims lacked registry coverage | Open as F4 |
| Required metadata was incomplete | Open and expanded with shell details as F5 |

## Scope

This product is a static site plus a local CLI. It has no backend, tenants,
account system, health endpoint, persistent server state, or product request
allowance. Backend tenant isolation, restart persistence, health, and
429/`Retry-After` tests do not apply. No deployment was requested or performed.
The brief does not need an AI feature; no missed AI step was found.
