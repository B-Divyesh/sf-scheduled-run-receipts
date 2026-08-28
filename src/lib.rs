//! Core receipt, schedule, and evidence primitives used by the `srr` CLI.
//!
//! Five-field schedules are evaluated in UTC:
//!
//! ```
//! use chrono::{DateTime, Utc};
//! use scheduled_run_receipts::{expected_between, parse_duration};
//!
//! let start: DateTime<Utc> = "2026-08-24T00:00:00Z".parse()?;
//! let end: DateTime<Utc> = "2026-08-25T03:00:00Z".parse()?;
//! let slots = expected_between("0 2 * * *", start, end)?;
//! assert_eq!(slots.len(), 2);
//! assert_eq!(parse_duration("15m")?.num_minutes(), 15);
//! # Ok::<(), anyhow::Error>(())
//! ```

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use cron::Schedule;
use fs2::FileExt;
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const RETENTION_DAYS: i64 = 7;
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u8,
    pub created_at: DateTime<Utc>,
    pub jobs: BTreeMap<String, Job>,
    pub receipts: Vec<Receipt>,
    #[serde(default)]
    pub seen_nonces: Vec<SeenNonce>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            created_at: Utc::now(),
            jobs: BTreeMap::new(),
            receipts: vec![],
            seen_nonces: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenNonce {
    pub value: String,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub schedule: String,
    pub grace_seconds: i64,
    pub secret: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Start,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FinishStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptPayload {
    pub version: u8,
    pub job: String,
    pub run_id: String,
    pub event: Event,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FinishStatus>,
    pub scheduled_at: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    #[serde(flatten)]
    pub payload: ReceiptPayload,
    pub accepted_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBundle {
    pub version: u8,
    pub job: String,
    pub schedule: String,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlotState {
    Success,
    Missing,
    Late,
    Failed,
    Running,
    Pending,
    Overlap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotResult {
    pub job: String,
    pub scheduled_at: DateTime<Utc>,
    pub state: SlotState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    pub generated_at: DateTime<Utc>,
    pub since: DateTime<Utc>,
    pub healthy: bool,
    pub counts: BTreeMap<String, usize>,
    pub slots: Vec<SlotResult>,
}

pub fn default_data_path() -> PathBuf {
    if let Ok(value) = std::env::var("SRR_DATA") {
        return PathBuf::from(value);
    }
    #[cfg(target_os = "windows")]
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(base).join("srr").join("state.json");
    }
    if let Ok(base) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(base).join("srr").join("state.json");
    }
    let base = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join(".local/share/srr/state.json")
}

pub fn load_state(path: &Path) -> Result<State> {
    let data = fs::read(path).with_context(|| {
        format!(
            "state not found at {}; run `srr init` first",
            path.display()
        )
    })?;
    let state: State = serde_json::from_slice(&data).context("state file is not valid JSON")?;
    if state.version != 1 {
        bail!("unsupported state version {}", state.version);
    }
    Ok(state)
}

pub fn save_state(path: &Path, state: &State) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // A unique temporary name prevents a direct library caller from colliding
    // with another writer. CLI mutations are additionally serialized by the
    // sibling lock file below.
    let temp = path.with_extension(format!("tmp-{}-{}", std::process::id(), random_nonce()));
    let bytes = serde_json::to_vec_pretty(state)?;
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    #[cfg(unix)]
    {
        // Persist the rename itself, not just the temporary file contents.
        fs::File::open(path.parent().unwrap_or_else(|| Path::new(".")))?.sync_all()?;
    }
    Ok(())
}

pub fn init(path: &Path, force: bool) -> Result<()> {
    with_state_lock(path, || {
        if path.exists() && !force {
            bail!(
                "state already exists at {}; pass --force to replace it",
                path.display()
            );
        }
        save_state(path, &State::default())
    })
}

/// Run one state transaction while holding an advisory, cross-process lock.
///
/// The lock is kept across load, mutation, and durable save so a command only
/// reports success after its receipt is present in the on-disk ledger.
pub fn mutate_state<T>(path: &Path, mutate: impl FnOnce(&mut State) -> Result<T>) -> Result<T> {
    with_state_lock(path, || {
        let mut state = load_state(path)?;
        let value = mutate(&mut state)?;
        save_state(path, &state)?;
        Ok(value)
    })
}

fn with_state_lock<T>(path: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let lock_path = path.with_extension("lock");
    let mut options = fs::OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options
        .open(&lock_path)
        .with_context(|| format!("cannot open state lock at {}", lock_path.display()))?;
    lock.lock_exclusive()
        .with_context(|| format!("cannot lock state at {}", lock_path.display()))?;
    let result = operation();
    FileExt::unlock(&lock).context("cannot unlock state")?;
    result
}

pub fn valid_job_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

pub fn parse_duration(value: &str) -> Result<Duration> {
    if value.len() < 2 {
        bail!("duration must look like 15m, 24h, or 7d");
    }
    let (number, unit) = value.split_at(value.len() - 1);
    let count: i64 = number
        .parse()
        .context("duration must start with a whole number")?;
    if count < 0 {
        bail!("duration cannot be negative");
    }
    let duration = match unit {
        "s" => Duration::try_seconds(count),
        "m" => Duration::try_minutes(count),
        "h" => Duration::try_hours(count),
        "d" => Duration::try_days(count),
        _ => bail!("duration unit must be s, m, h, or d"),
    };
    duration.ok_or_else(|| anyhow!("duration is too large"))
}

pub fn parse_schedule(expression: &str) -> Result<Schedule> {
    if expression.split_whitespace().count() != 5 {
        bail!("schedule must be a five-field cron expression in UTC");
    }
    Schedule::from_str(&format!("0 {expression} *")).context("invalid cron schedule")
}

pub fn expected_between(
    expression: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<DateTime<Utc>>> {
    let schedule = parse_schedule(expression)?;
    Ok(schedule
        .after(&(start - Duration::seconds(1)))
        .take_while(|date| *date <= end)
        .collect())
}

pub fn latest_expected(expression: &str, at: DateTime<Utc>) -> Result<DateTime<Utc>> {
    parse_schedule(expression)?
        .after(&(at - Duration::days(731)))
        .take_while(|date| *date <= at)
        .last()
        .ok_or_else(|| anyhow!("schedule has no occurrence in the prior two years"))
}

pub fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn random_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn sign_payload(payload: &ReceiptPayload, secret: &str) -> Result<String> {
    let body = serde_json::to_vec(payload)?;
    let key = URL_SAFE_NO_PAD
        .decode(secret)
        .context("invalid receipt key")?;
    let mut mac = HmacSha256::new_from_slice(&key).map_err(|_| anyhow!("invalid receipt key"))?;
    mac.update(&body);
    let signature = mac.finalize().into_bytes();
    Ok(format!(
        "srr1.{}.{}",
        URL_SAFE_NO_PAD.encode(body),
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

pub fn decode_token(token: &str) -> Result<(ReceiptPayload, Vec<u8>, Vec<u8>)> {
    let mut parts = token.split('.');
    if parts.next() != Some("srr1") {
        bail!("unsupported receipt token");
    }
    let body = URL_SAFE_NO_PAD
        .decode(
            parts
                .next()
                .ok_or_else(|| anyhow!("receipt payload missing"))?,
        )
        .context("invalid receipt payload encoding")?;
    let signature = URL_SAFE_NO_PAD
        .decode(
            parts
                .next()
                .ok_or_else(|| anyhow!("receipt signature missing"))?,
        )
        .context("invalid receipt signature encoding")?;
    if parts.next().is_some() {
        bail!("receipt token has extra fields");
    }
    let payload = serde_json::from_slice(&body).context("invalid receipt payload")?;
    Ok((payload, body, signature))
}

pub fn accept_token(state: &mut State, token: &str, now: DateTime<Utc>) -> Result<Receipt> {
    let (payload, body, signature) = decode_token(token)?;
    if payload.version != 1 {
        bail!("unsupported receipt version");
    }
    let job = state
        .jobs
        .get(&payload.job)
        .ok_or_else(|| anyhow!("unknown job `{}`", payload.job))?;
    let key = URL_SAFE_NO_PAD
        .decode(&job.secret)
        .context("stored job key is invalid")?;
    let mut mac =
        HmacSha256::new_from_slice(&key).map_err(|_| anyhow!("stored job key is invalid"))?;
    mac.update(&body);
    mac.verify_slice(&signature)
        .map_err(|_| anyhow!("receipt signature did not verify"))?;
    if payload.occurred_at < now - Duration::days(RETENTION_DAYS) {
        bail!("receipt is older than the 7-day acceptance window");
    }
    if payload.occurred_at > now + Duration::minutes(5) {
        bail!("receipt time is more than 5 minutes in the future");
    }
    if state
        .seen_nonces
        .iter()
        .any(|nonce| nonce.value == payload.nonce)
        || state.receipts.iter().any(|receipt| {
            receipt.payload.nonce == payload.nonce
                && receipt.accepted_at >= now - Duration::days(RETENTION_DAYS)
        })
    {
        bail!("receipt replay rejected: nonce was already accepted");
    }
    match (&payload.event, &payload.status) {
        (Event::Start, None) | (Event::Finish, Some(_)) => {}
        (Event::Start, Some(_)) => bail!("start receipts cannot include a finish status"),
        (Event::Finish, None) => bail!("finish receipts require a status"),
    }
    let receipt = Receipt {
        payload,
        accepted_at: now,
        signature: URL_SAFE_NO_PAD.encode(signature),
    };
    state.seen_nonces.push(SeenNonce {
        value: receipt.payload.nonce.clone(),
        accepted_at: now,
    });
    state.receipts.push(receipt.clone());
    state
        .seen_nonces
        .retain(|nonce| nonce.accepted_at >= now - Duration::days(RETENTION_DAYS));
    Ok(receipt)
}

pub fn add_job(state: &mut State, name: &str, schedule: &str, grace: Duration) -> Result<()> {
    if !valid_job_name(name) {
        bail!("job name must use lowercase letters, digits, - or _ (max 64 characters)");
    }
    parse_schedule(schedule)?;
    if grace < Duration::zero() {
        bail!("grace cannot be negative");
    }
    if state.jobs.contains_key(name) {
        bail!("job `{name}` already exists");
    }
    state.jobs.insert(
        name.into(),
        Job {
            name: name.into(),
            schedule: schedule.into(),
            grace_seconds: grace.num_seconds(),
            secret: random_secret(),
            created_at: Utc::now(),
        },
    );
    Ok(())
}

pub fn issue_payload(
    job: &Job,
    event: Event,
    run_id: &str,
    scheduled_at: Option<DateTime<Utc>>,
    status: Option<FinishStatus>,
    now: DateTime<Utc>,
) -> Result<ReceiptPayload> {
    if run_id.trim().is_empty() || run_id.len() > 128 {
        bail!("run ID must be 1–128 characters");
    }
    let scheduled_at = match scheduled_at {
        Some(value) => value,
        None => latest_expected(&job.schedule, now)?,
    };
    match (&event, &status) {
        (Event::Start, None) | (Event::Finish, Some(_)) => {}
        (Event::Start, Some(_)) => bail!("start receipts cannot include a finish status"),
        (Event::Finish, None) => bail!("finish receipts require --status"),
    }
    Ok(ReceiptPayload {
        version: 1,
        job: job.name.clone(),
        run_id: run_id.into(),
        event,
        status,
        scheduled_at,
        occurred_at: now,
        nonce: random_nonce(),
    })
}

pub fn check(state: &State, since: DateTime<Utc>, now: DateTime<Utc>) -> Result<CheckReport> {
    let mut slots = vec![];
    let mut overlapping: HashSet<(String, DateTime<Utc>)> = HashSet::new();

    for job in state.jobs.values() {
        let starts: Vec<&Receipt> = state
            .receipts
            .iter()
            .filter(|r| r.payload.job == job.name && r.payload.event == Event::Start)
            .collect();
        let mut ordered = starts.clone();
        ordered.sort_by_key(|r| r.payload.occurred_at);
        for pair in ordered.windows(2) {
            let current = pair[0];
            let next = pair[1];
            let finish = state
                .receipts
                .iter()
                .filter(|r| {
                    r.payload.job == job.name
                        && r.payload.run_id == current.payload.run_id
                        && r.payload.event == Event::Finish
                })
                .map(|r| r.payload.occurred_at)
                .min();
            if finish.is_none_or(|end| next.payload.occurred_at < end) {
                overlapping.insert((job.name.clone(), next.payload.scheduled_at));
            }
        }

        for expected in expected_between(&job.schedule, since, now)? {
            let start = starts
                .iter()
                .copied()
                .filter(|r| r.payload.scheduled_at == expected)
                .min_by_key(|r| r.payload.occurred_at);
            let (state_name, run_id, started_at, finished_at) = if let Some(start) = start {
                let finish = state
                    .receipts
                    .iter()
                    .filter(|r| {
                        r.payload.job == job.name
                            && r.payload.run_id == start.payload.run_id
                            && r.payload.event == Event::Finish
                    })
                    .min_by_key(|r| r.payload.occurred_at);
                let overlap = overlapping.contains(&(job.name.clone(), expected));
                let late =
                    start.payload.occurred_at > expected + Duration::seconds(job.grace_seconds);
                let state_name = if overlap {
                    SlotState::Overlap
                } else if finish.is_some_and(|r| r.payload.status == Some(FinishStatus::Failure)) {
                    SlotState::Failed
                } else if finish.is_none() {
                    SlotState::Running
                } else if late {
                    SlotState::Late
                } else {
                    SlotState::Success
                };
                (
                    state_name,
                    Some(start.payload.run_id.clone()),
                    Some(start.payload.occurred_at),
                    finish.map(|r| r.payload.occurred_at),
                )
            } else if now > expected + Duration::seconds(job.grace_seconds) {
                (SlotState::Missing, None, None, None)
            } else {
                (SlotState::Pending, None, None, None)
            };
            slots.push(SlotResult {
                job: job.name.clone(),
                scheduled_at: expected,
                state: state_name,
                run_id,
                started_at,
                finished_at,
            });
        }
    }
    slots.sort_by_key(|slot| (slot.scheduled_at, slot.job.clone()));
    let mut counts = BTreeMap::new();
    for slot in &slots {
        *counts
            .entry(format!("{:?}", slot.state).to_lowercase())
            .or_insert(0) += 1;
    }
    let healthy = slots
        .iter()
        .all(|slot| matches!(slot.state, SlotState::Success | SlotState::Pending));
    Ok(CheckReport {
        generated_at: now,
        since,
        healthy,
        counts,
        slots,
    })
}

pub fn week_bounds(
    value: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
    let date = match value {
        Some(v) => NaiveDate::parse_from_str(v, "%Y-%m-%d").context("week must be YYYY-MM-DD")?,
        None => now.date_naive(),
    };
    let monday = date - Duration::days(date.weekday().num_days_from_monday() as i64);
    let start = Utc.from_utc_datetime(&monday.and_hms_opt(0, 0, 0).unwrap());
    Ok((start, start + Duration::days(7) - Duration::seconds(1)))
}

pub fn evidence_digest(report: &CheckReport) -> Result<String> {
    let bytes = serde_json::to_vec(report)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn export_html(
    report: &CheckReport,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<String> {
    let digest = evidence_digest(report)?;
    let mut rows = String::new();
    for slot in &report.slots {
        let state = format!("{:?}", slot.state);
        let state_class = state.to_lowercase();
        rows.push_str(&format!("<tr><td>{}</td><td><time datetime=\"{}\">{}</time></td><td><span class=\"mark {}\">{}</span></td><td>{}</td></tr>",
            html_escape(&slot.job), slot.scheduled_at.to_rfc3339(), slot.scheduled_at.format("%a %H:%M UTC"),
            state_class, state, html_escape(slot.run_id.as_deref().unwrap_or("—"))));
    }
    if rows.is_empty() {
        rows.push_str(
            "<tr><td colspan=\"4\" class=\"empty\">No expected runs in this week.</td></tr>",
        );
    }
    let result = if report.healthy {
        "All due runs accounted for"
    } else {
        "Exceptions require review"
    };
    Ok(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Run receipts — {week}</title><style>
:root{{--ink:#17262c;--paper:#f5f1e7;--line:#c8c2b4;--ok:#176b4b;--bad:#8f2415;--warn:#765500}}*{{box-sizing:border-box}}body{{margin:0;background:#ddd7c9;color:var(--ink);font:16px/1.5 ui-sans-serif,system-ui,sans-serif}}main{{max-width:1000px;margin:32px auto;background:var(--paper);padding:clamp(24px,6vw,64px);box-shadow:0 12px 40px #17262c25}}header{{display:grid;grid-template-columns:1fr auto;gap:24px;border-bottom:2px solid var(--ink);padding-bottom:24px}}h1{{font-size:clamp(2rem,6vw,4rem);line-height:.95;letter-spacing:-.045em;margin:8px 0}}.eyebrow,.digest{{font:700 .75rem/1.4 ui-monospace,monospace;letter-spacing:.1em;text-transform:uppercase}}.seal{{border:2px solid var(--ink);width:112px;height:112px;border-radius:50%;display:grid;place-items:center;text-align:center;font-weight:800}}.summary{{margin:40px 0;display:flex;justify-content:space-between;gap:24px;align-items:end}}.summary strong{{font-size:1.5rem}}table{{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums}}th,td{{padding:14px 8px;text-align:left;border-bottom:1px solid var(--line)}}th{{font-size:.75rem;text-transform:uppercase;letter-spacing:.08em}}.mark{{font-weight:800;text-transform:uppercase;font-size:.78rem}}.success{{color:var(--ok)}}.missing,.failed,.overlap{{color:var(--bad)}}.late,.running,.pending{{color:var(--warn)}}.digest{{overflow-wrap:anywhere;margin-top:40px;padding-top:16px;border-top:1px solid var(--line)}}.empty{{text-align:center;padding:48px}}@media(max-width:600px){{main{{margin:0;min-height:100vh}}header{{grid-template-columns:1fr}}.seal{{width:88px;height:88px}}th:nth-child(4),td:nth-child(4){{display:none}}}}@media print{{body{{background:white}}main{{margin:0;box-shadow:none;max-width:none}}}}</style></head><body><main><header><div><div class="eyebrow">Scheduled Run Receipts · Weekly evidence</div><h1>Run ledger</h1><p>{start} through {end}</p></div><div class="seal">LOCAL<br>EVIDENCE</div></header><section class="summary" aria-labelledby="result"><div><div class="eyebrow">Result</div><strong id="result">{result}</strong></div><div>{count} expected slots</div></section><table><thead><tr><th>Job</th><th>Expected</th><th>Finding</th><th>Run ID</th></tr></thead><tbody>{rows}</tbody></table><p class="digest">SHA-256 evidence digest<br>{digest}</p><p>Generated {generated}. This standalone report contains no scripts or remote resources.</p></main></body></html>"#,
        week = start.format("%Y-%m-%d"),
        start = start.format("%d %b %Y"),
        end = end.format("%d %b %Y"),
        count = report.slots.len(),
        generated = report.generated_at.to_rfc3339()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }

    #[test]
    fn parses_documented_durations_and_cron() {
        assert_eq!(parse_duration("15m").unwrap(), Duration::minutes(15));
        assert!(parse_duration("15x").is_err());
        let times = expected_between(
            "0 2 * * *",
            dt("2026-08-24T00:00:00Z"),
            dt("2026-08-25T03:00:00Z"),
        )
        .unwrap();
        assert_eq!(times.len(), 2);
        assert_eq!(times[0], dt("2026-08-24T02:00:00Z"));
    }

    #[test]
    fn signed_receipt_verifies_and_replay_fails() {
        let now = dt("2026-08-28T02:02:00Z");
        let mut state = State::default();
        add_job(&mut state, "backup", "0 2 * * *", Duration::minutes(15)).unwrap();
        let job = state.jobs["backup"].clone();
        let payload = issue_payload(&job, Event::Start, "r1", None, None, now).unwrap();
        let token = sign_payload(&payload, &job.secret).unwrap();
        accept_token(&mut state, &token, now).unwrap();
        assert!(
            accept_token(&mut state, &token, now)
                .unwrap_err()
                .to_string()
                .contains("replay")
        );
    }

    #[test]
    fn rejects_tampered_and_stale_receipts() {
        let now = dt("2026-08-28T02:02:00Z");
        let mut state = State::default();
        add_job(&mut state, "backup", "0 2 * * *", Duration::minutes(15)).unwrap();
        let job = state.jobs["backup"].clone();
        let payload = issue_payload(&job, Event::Start, "r1", None, None, now).unwrap();
        let token = sign_payload(&payload, &job.secret).unwrap();
        let mut tampered = token.into_bytes();
        let last = tampered.len() - 1;
        tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
        assert!(accept_token(&mut state, std::str::from_utf8(&tampered).unwrap(), now).is_err());

        let stale_time = now - Duration::days(8);
        let stale = issue_payload(&job, Event::Start, "old", None, None, stale_time).unwrap();
        let stale_token = sign_payload(&stale, &job.secret).unwrap();
        assert!(
            accept_token(&mut state, &stale_token, now)
                .unwrap_err()
                .to_string()
                .contains("older")
        );
    }

    #[test]
    fn detects_success_late_missing_and_overlap() {
        let now = dt("2026-08-28T03:00:00Z");
        let mut state = State::default();
        add_job(&mut state, "backup", "0 2 * * *", Duration::minutes(15)).unwrap();
        let job = state.jobs["backup"].clone();
        for (run, scheduled, started, finished) in [
            (
                "good",
                "2026-08-25T02:00:00Z",
                "2026-08-25T02:02:00Z",
                Some("2026-08-25T02:03:00Z"),
            ),
            (
                "late",
                "2026-08-26T02:00:00Z",
                "2026-08-26T02:20:00Z",
                Some("2026-08-26T02:21:00Z"),
            ),
            ("open", "2026-08-27T02:00:00Z", "2026-08-27T02:01:00Z", None),
            (
                "overlap",
                "2026-08-28T02:00:00Z",
                "2026-08-28T02:01:00Z",
                None,
            ),
        ] {
            let mut start = issue_payload(
                &job,
                Event::Start,
                run,
                Some(dt(scheduled)),
                None,
                dt(started),
            )
            .unwrap();
            start.occurred_at = dt(started);
            let token = sign_payload(&start, &job.secret).unwrap();
            accept_token(&mut state, &token, dt(started)).unwrap();
            if let Some(finished) = finished {
                let finish = issue_payload(
                    &job,
                    Event::Finish,
                    run,
                    Some(dt(scheduled)),
                    Some(FinishStatus::Success),
                    dt(finished),
                )
                .unwrap();
                let token = sign_payload(&finish, &job.secret).unwrap();
                accept_token(&mut state, &token, dt(finished)).unwrap();
            }
        }
        let report = check(&state, dt("2026-08-25T00:00:00Z"), now).unwrap();
        assert_eq!(
            report
                .slots
                .iter()
                .find(|s| s.scheduled_at == dt("2026-08-25T02:00:00Z"))
                .unwrap()
                .state,
            SlotState::Success
        );
        assert_eq!(
            report
                .slots
                .iter()
                .find(|s| s.scheduled_at == dt("2026-08-26T02:00:00Z"))
                .unwrap()
                .state,
            SlotState::Late
        );
        assert_eq!(
            report
                .slots
                .iter()
                .find(|s| s.scheduled_at == dt("2026-08-27T02:00:00Z"))
                .unwrap()
                .state,
            SlotState::Running
        );
        assert_eq!(
            report
                .slots
                .iter()
                .find(|s| s.scheduled_at == dt("2026-08-28T02:00:00Z"))
                .unwrap()
                .state,
            SlotState::Overlap
        );
        assert!(!report.healthy);
    }

    #[test]
    fn export_is_standalone_and_escaped() {
        let report = CheckReport {
            generated_at: dt("2026-08-28T03:00:00Z"),
            since: dt("2026-08-24T00:00:00Z"),
            healthy: false,
            counts: BTreeMap::new(),
            slots: vec![SlotResult {
                job: "backup".into(),
                scheduled_at: dt("2026-08-25T02:00:00Z"),
                state: SlotState::Missing,
                run_id: None,
                started_at: None,
                finished_at: None,
            }],
        };
        let html = export_html(
            &report,
            dt("2026-08-24T00:00:00Z"),
            dt("2026-08-30T23:59:59Z"),
        )
        .unwrap();
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("SHA-256 evidence digest"));
        assert!(!html.contains("<script"));
    }
}
