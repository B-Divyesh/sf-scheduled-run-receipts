# Independent product verification — FAIL

**Candidate:** `e7d14f5c7db523842ddc628e8d4316614e634b79`

**Live URL:** <https://scheduled-run-receipts.sociobot.in>

**Verified:** 2026-08-28 UTC

**Acceptance contract:** `.factory/brief.json`, repository `AGENTS.md`, and the supplied factory work order

**Disposition:** **FAIL**

The deployment is healthy and exactly matches the candidate, but the candidate
cannot safely preserve its core evidence when independent cron processes write
at the same time. A 20-process burst produced 16 success messages and four
errors while only one receipt survived in the state file. This is a
release-blocking failure for a receipt ledger intended to monitor multiple
scheduled jobs.

## Defects

### High — concurrent writers silently lose accepted receipts

The state store has no cross-process lock. Every command loads the whole JSON
file and writes through the same `state.json.tmp` path. In a clean state with
one valid per-minute job, 20 simultaneous `srr run start` commands gave:

```text
attempts=20
commands printing "Accepted Start"=16
commands returning an error=4
persisted receipts=1
persisted nonces=1
error from four commands="No such file or directory (os error 2)"
```

Only `run-10` remained. Fifteen commands falsely reported success even though
their evidence was lost. This violates the primary job-to-be-done and makes
overlapping jobs unsafe even on one host.

Reproduction with an installed release package:

```sh
srr --data state.json init
srr --data state.json job add concurrent --schedule '* * * * *' --grace 1m
for i in $(seq 1 20); do
  srr --data state.json run start concurrent --run-id "run-$i" &
done
wait
jq '.receipts | length' state.json
```

### Medium — validly parsed extreme durations panic

Boundary values that fit `i64` are parsed, then panic inside `chrono` rather
than returning the documented input-error exit code:

```text
--grace 9223372036854775807s -> exit 101, TimeDelta::seconds out of bounds
--grace 9223372036854775807d -> exit 101, TimeDelta::days out of bounds
```

Ordinary invalid values recover cleanly: `--grace=-1s` and `1x` return exit 1
with actionable messages, and the state remains valid.

### Medium — page has material horizontal overflow

Browser measurements after network idle:

| Viewport | `clientWidth` | `scrollWidth` | Overflow |
| --- | ---: | ---: | ---: |
| Desktop | 1440 | 1487 | 47 px |
| Mobile | 390 | 598 | 208 px |

The absolutely positioned hero artwork expands the document beyond the
viewport. At 390 px this creates a large unintended horizontal pan and fails
the mobile acceptance requirement.

### Low — interaction, delivery, and quality issues

- The ARIA radio group responds to Left/Right and updates its detail, but all
  seven radios remain in the Tab order instead of using one roving tab stop.
- Several visible links have hit boxes only about 24.8 px high (brand, plain
  navigation, and footer links), below the contract's 44 × 44 px target.
- Every hosted file, including hashed JS/CSS/font/image assets, is served with
  `Cache-Control: public, must-revalidate, max-age=30`; hashed assets are not
  long-lived/immutable as required. Conditional ETag requests do return 304.
- The response has HSTS, `nosniff`, referrer policy, and DNS-prefetch policy,
  but no response-level CSP, `frame-ancestors`/X-Frame-Options, or
  Permissions-Policy. The main page has a restrictive meta CSP; legal pages do
  not. The HSTS `preload` directive is paired with a 10,886,400-second max age,
  below preload-list eligibility.
- An unknown route such as `/definitely-missing-qa` returns the landing page
  with HTTP 200 rather than a 404.
- `cargo clippy --all-targets --all-features -- -D warnings` fails on
  `collapsible_if` and `format_in_format_args`. Formatting passes. The site has
  no configured lint/typecheck script or `tsconfig.json`; an ad hoc strict
  `tsc` invocation is not runnable without the missing Node/Vite type setup.

## Clean checkout and build evidence

Verification used a clean detached clone at the exact candidate. The clone was
clean before and after the gates. Toolchain: Node `v22.23.2`, npm `10.9.8`,
rustc `1.98.0`, Cargo `1.98.0`.

| Gate | Result |
| --- | --- |
| `npm ci` | PASS — 61 packages installed; 0 vulnerabilities |
| `npm test` | PASS |
| Rust unit tests | PASS — 5/5 |
| Rust process-level CLI tests | PASS — 2/2 |
| Rust doctests | PASS — 1/1 |
| `npm run test:site` | PASS — 2/2 |
| Playwright suite | PASS — 4/4 across desktop and 390 × 844 |
| `cargo fmt --all -- --check` | PASS |
| strict Cargo Clippy | FAIL — two warning-level findings listed above |
| `npm run build` | PASS — exact production command |
| `cargo package --locked` | PASS — 12 files, 22.4 KiB compressed, package verification compiled |
| `npm audit --audit-level=low` | PASS — 0 vulnerabilities |

The package's `.cargo_vcs_info.json` names the candidate SHA. Installing the
packaged crate into a new consumer root with `cargo install --locked --path
target/package/scheduled-run-receipts-0.1.0` succeeded; the installed binary
reported `srr 0.1.0` and its help exposed the documented commands and global
data-path option.

Production output is 159,761 bytes total. Raw budgets pass:

- JavaScript: 5,722 bytes (2,440 bytes transferred in Lighthouse)
- CSS: 10,848 bytes (3,430 bytes transferred)
- Emitted fonts: 69,280 bytes; browser loaded 45,101 bytes
- Hero WebP: 61,360 bytes
- Total Lighthouse transfer: 115,600 bytes, with zero third-party bytes

## CLI and library behavior

Representative clean-consumer workflows passed apart from concurrency and the
extreme-duration panic:

- `init`, add/list/key/rotate-capable command surface, local start/finish,
  portable sign/accept, status, check, and weekly export all ran from the
  installed package.
- State and exported key files were mode `0600` on Linux.
- A success receipt appeared in `status --json`; an independent per-minute
  scenario classified exactly one each of `late`, `failed`, `missing`,
  `running`, and `overlap`, with `check` returning exit 2.
- A repeated signed token and a signature-tampered token both returned exit 1;
  the replay error identified the reused nonce.
- A 64-character job name was accepted; 65 characters, uppercase input, a
  six-field cron, empty run IDs, and 129-character run IDs were rejected with
  actionable errors.
- Corrupt JSON returned exit 1 with its parse location; `init --force`
  recovered to an empty valid mode-0600 store.
- The generated weekly evidence page was standalone and script-free, escaped
  untrusted values, and contained one 64-hex SHA-256 digest.
- A job secret occurred once in local state and did not occur in check JSON or
  exported HTML. Source/dependency inspection found no CLI network client,
  analytics, telemetry, or background process.
- The documented public Rust example compiled as a doctest.

## Live deployment, browser, privacy, and PWA evidence

Freshly built `dist/site` contained 14 files. Every live path had the same
SHA-256 digest as its local candidate artifact; there were **0 mismatches**.
The root HTML digest on both sides was
`024b289365b75ba67b1c302075459b5d68a5905d312e6b798b6558d7b22471ba`.
This rules out the previously suspected deployment-only failure.

- HTTPS root, privacy, terms, service worker, robots, sitemap, and all assets
  returned 200. HTTP redirects to HTTPS with 301.
- Factory URL verification: 200, 898 ms network-idle load, title and `lang`
  present, exactly one `h1`, `main` present, all images have alt text, no
  unlabeled buttons, and no console/page errors.
- Independent Playwright at 1440 × 1000 and 390 × 844: no console or page
  errors and zero axe serious/critical findings.
- Keyboard smoke test: the first Tab exposes the skip link with a 3 px chalk
  outline plus 6 px vermilion ring; Enter/Space work on native controls; radio
  Left/Right selection and live detail updates work. The seven-tab-stop issue
  is recorded above.
- Invalid report JSON gives an actionable error, and selecting a valid empty
  report immediately recovers to the empty state. The status says explicitly
  that the file was loaded locally.
- All six normal initial requests used only
  `scheduled-run-receipts.sociobot.in`. There are no cookies, analytics,
  remote fonts, third-party scripts, payment calls, unlock calls, or sign-in.
- `prefers-reduced-motion: reduce` changes smooth scroll to `auto` and reduces
  transitions to effectively instant; nothing loops or flashes.
- The service worker registered at root, `registration.update()` completed
  with the candidate worker active and no waiting/installing worker, and a
  subsequent offline reload retained the correct title and visible main
  content.
- Mobile Lighthouse 13.0.1: performance **99**, accessibility **100**, best
  practices **100**, SEO **100**; FCP 1.1 s, LCP 1.4 s, TBT 90 ms, CLS 0.054,
  speed index 1.1 s, interactive 1.4 s.

Rate limiting is not applicable: the product is a static PWA plus a local CLI,
and browser/source inspection found no product API, unlock endpoint, or other
server-side endpoint. Authentication/Entra verification is likewise not
applicable because the product has no account or sign-in. Backend concurrency,
health/build-identity endpoints, and server persistence are not applicable;
the relevant local cross-process persistence boundary was tested and failed as
documented above.

## Required remediation before re-verification

1. Add a real cross-process transaction/locking strategy and acknowledge
   success only after the caller's receipt is durably present. Add a
   concurrent-process regression test.
2. Bound duration values and convert overflow to a normal exit-1 input error.
3. Clip or constrain the hero so the document width never exceeds the
   viewport, including 390 px.
4. Correct radio roving focus and 44 px hit areas; configure immutable caching
   for hashed assets and strengthen response headers.
5. Make strict Clippy pass and add a repository-owned TypeScript typecheck.
