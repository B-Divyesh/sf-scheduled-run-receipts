use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use scheduled_run_receipts::{
    Event, FinishStatus, Job, KeyBundle, SlotState, accept_token, add_job, check,
    default_data_path, export_html, init, issue_payload, load_state, parse_duration, random_secret,
    save_state, sign_payload, week_bounds,
};

#[derive(Parser)]
#[command(name = "srr", version, about = "Prove scheduled runs happened — and notice when they did not", long_about = None)]
struct Cli {
    /// State file path (or set SRR_DATA)
    #[arg(long, global = true, env = "SRR_DATA")]
    data: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create an empty local receipt store
    Init {
        #[arg(long)]
        force: bool,
    },
    /// Register and manage expected jobs
    Job {
        #[command(subcommand)]
        command: JobCommand,
    },
    /// Record a start or finish directly in the local store
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Issue or accept a portable signed receipt
    Receipt {
        #[command(subcommand)]
        command: ReceiptCommand,
    },
    /// Compare accepted receipts with the expected calendar
    Check {
        #[arg(long, default_value = "7d")]
        since: String,
        #[arg(long)]
        json: bool,
    },
    /// Show the last known successful run for each job
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Write a standalone weekly HTML evidence page
    Export {
        #[arg(long)]
        week: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum JobCommand {
    /// Add a job and generate its independent signing key
    Add {
        job: String,
        #[arg(long)]
        schedule: String,
        #[arg(long, default_value = "15m")]
        grace: String,
    },
    /// List configured jobs without exposing keys
    List {
        #[arg(long)]
        json: bool,
    },
    /// Export one job's runner key bundle
    Key {
        job: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        force: bool,
    },
    /// Replace a job's signing key; previously issued tokens stop verifying
    RotateKey { job: String },
}

#[derive(Subcommand)]
enum RunCommand {
    /// Record a signed start receipt locally
    Start(RunStart),
    /// Record a signed finish receipt locally
    Finish(RunFinish),
}

#[derive(Args)]
struct RunStart {
    job: String,
    #[arg(long)]
    run_id: String,
    #[arg(long, value_parser = parse_datetime)]
    scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Args)]
struct RunFinish {
    job: String,
    #[arg(long)]
    run_id: String,
    #[arg(long, value_enum)]
    status: StatusArg,
}

#[derive(Subcommand)]
enum ReceiptCommand {
    /// Create a portable token without touching monitor state
    Sign {
        #[arg(value_enum)]
        event: EventArg,
        #[arg(long)]
        job: String,
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        key_file: PathBuf,
        #[arg(long, value_parser = parse_datetime)]
        scheduled_at: Option<DateTime<Utc>>,
        #[arg(long, value_enum)]
        status: Option<StatusArg>,
    },
    /// Verify and store a token; duplicate nonces are rejected
    Accept { token: String },
}

#[derive(Clone, ValueEnum)]
enum EventArg {
    Start,
    Finish,
}
#[derive(Clone, ValueEnum)]
enum StatusArg {
    Success,
    Failure,
}
impl From<EventArg> for Event {
    fn from(v: EventArg) -> Self {
        match v {
            EventArg::Start => Event::Start,
            EventArg::Finish => Event::Finish,
        }
    }
}
impl From<StatusArg> for FinishStatus {
    fn from(v: StatusArg) -> Self {
        match v {
            StatusArg::Success => FinishStatus::Success,
            StatusArg::Failure => FinishStatus::Failure,
        }
    }
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, String> {
    value
        .parse::<DateTime<Utc>>()
        .map_err(|_| "expected RFC3339 UTC time such as 2026-08-28T02:00:00Z".into())
}

fn write_private(path: &Path, bytes: &[u8], force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to replace it",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    std::io::Write::write_all(&mut options.open(path)?, bytes)?;
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<u8> {
    let path = cli.data.unwrap_or_else(default_data_path);
    match cli.command {
        Command::Init { force } => {
            init(&path, force)?;
            println!("Initialized local receipt store at {}", path.display());
        }
        Command::Job { command } => {
            let mut state = load_state(&path)?;
            match command {
                JobCommand::Add {
                    job,
                    schedule,
                    grace,
                } => {
                    add_job(&mut state, &job, &schedule, parse_duration(&grace)?)?;
                    save_state(&path, &state)?;
                    println!("Added {job}: {schedule} UTC, grace {grace}");
                }
                JobCommand::List { json } => {
                    #[derive(serde::Serialize)]
                    struct PublicJob<'a> {
                        name: &'a str,
                        schedule: &'a str,
                        grace_seconds: i64,
                    }
                    let jobs: Vec<_> = state
                        .jobs
                        .values()
                        .map(|j| PublicJob {
                            name: &j.name,
                            schedule: &j.schedule,
                            grace_seconds: j.grace_seconds,
                        })
                        .collect();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&jobs)?);
                    } else if jobs.is_empty() {
                        println!("No jobs configured. Add one with `srr job add`.");
                    } else {
                        for job in jobs {
                            println!(
                                "{}  {} UTC  grace {}s",
                                job.name, job.schedule, job.grace_seconds
                            );
                        }
                    }
                }
                JobCommand::Key { job, output, force } => {
                    let job = state
                        .jobs
                        .get(&job)
                        .ok_or_else(|| anyhow!("unknown job `{job}`"))?;
                    let bundle = KeyBundle {
                        version: 1,
                        job: job.name.clone(),
                        schedule: job.schedule.clone(),
                        secret: job.secret.clone(),
                    };
                    write_private(&output, &serde_json::to_vec_pretty(&bundle)?, force)?;
                    println!("Wrote runner key for {} to {}", job.name, output.display());
                }
                JobCommand::RotateKey { job } => {
                    let target = state
                        .jobs
                        .get_mut(&job)
                        .ok_or_else(|| anyhow!("unknown job `{job}`"))?;
                    target.secret = random_secret();
                    save_state(&path, &state)?;
                    println!("Rotated key for {job}; export a new runner key before the next run");
                }
            }
        }
        Command::Run { command } => {
            let mut state = load_state(&path)?;
            let now = Utc::now();
            let (job_name, payload) = match command {
                RunCommand::Start(args) => {
                    let job = state
                        .jobs
                        .get(&args.job)
                        .ok_or_else(|| anyhow!("unknown job `{}`", args.job))?;
                    (
                        job.name.clone(),
                        issue_payload(
                            job,
                            Event::Start,
                            &args.run_id,
                            args.scheduled_at,
                            None,
                            now,
                        )?,
                    )
                }
                RunCommand::Finish(args) => {
                    let job = state
                        .jobs
                        .get(&args.job)
                        .ok_or_else(|| anyhow!("unknown job `{}`", args.job))?;
                    let scheduled = state
                        .receipts
                        .iter()
                        .rev()
                        .find(|r| {
                            r.payload.job == args.job
                                && r.payload.run_id == args.run_id
                                && r.payload.event == Event::Start
                        })
                        .map(|r| r.payload.scheduled_at)
                        .ok_or_else(|| {
                            anyhow!("no accepted start receipt for run `{}`", args.run_id)
                        })?;
                    (
                        job.name.clone(),
                        issue_payload(
                            job,
                            Event::Finish,
                            &args.run_id,
                            Some(scheduled),
                            Some(args.status.into()),
                            now,
                        )?,
                    )
                }
            };
            let secret = state.jobs[&job_name].secret.clone();
            let token = sign_payload(&payload, &secret)?;
            let receipt = accept_token(&mut state, &token, now)?;
            save_state(&path, &state)?;
            println!(
                "Accepted {:?} receipt for {} / {} ({})",
                receipt.payload.event,
                receipt.payload.job,
                receipt.payload.run_id,
                receipt.payload.scheduled_at.to_rfc3339()
            );
        }
        Command::Receipt { command } => match command {
            ReceiptCommand::Sign {
                event,
                job,
                run_id,
                key_file,
                scheduled_at,
                status,
            } => {
                let bundle: KeyBundle = serde_json::from_slice(
                    &fs::read(&key_file)
                        .with_context(|| format!("cannot read {}", key_file.display()))?,
                )
                .context("key file is invalid")?;
                if bundle.version != 1 {
                    bail!("unsupported key file version");
                }
                if bundle.job != job {
                    bail!("key file belongs to `{}`, not `{job}`", bundle.job);
                }
                let local_job = Job {
                    name: bundle.job,
                    schedule: bundle.schedule,
                    grace_seconds: 0,
                    secret: bundle.secret.clone(),
                    created_at: Utc::now(),
                };
                let payload = issue_payload(
                    &local_job,
                    event.into(),
                    &run_id,
                    scheduled_at,
                    status.map(Into::into),
                    Utc::now(),
                )?;
                println!("{}", sign_payload(&payload, &bundle.secret)?);
            }
            ReceiptCommand::Accept { token } => {
                let mut state = load_state(&path)?;
                let receipt = accept_token(&mut state, &token, Utc::now())?;
                save_state(&path, &state)?;
                println!(
                    "Accepted {:?} receipt for {} / {}",
                    receipt.payload.event, receipt.payload.job, receipt.payload.run_id
                );
            }
        },
        Command::Check { since, json } => {
            let state = load_state(&path)?;
            let now = Utc::now();
            let report = check(&state, now - parse_duration(&since)?, now)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else if state.jobs.is_empty() {
                println!("No jobs configured. Add one with `srr job add`.");
            } else {
                println!(
                    "{} expected slots · {}",
                    report.slots.len(),
                    if report.healthy {
                        "healthy"
                    } else {
                        "exceptions found"
                    }
                );
                for slot in report
                    .slots
                    .iter()
                    .filter(|s| !matches!(s.state, SlotState::Success | SlotState::Pending))
                {
                    println!(
                        "{:?}  {}  {}",
                        slot.state,
                        slot.job,
                        slot.scheduled_at.to_rfc3339()
                    );
                }
            }
            if !report.healthy {
                return Ok(2);
            }
        }
        Command::Status { json } => {
            let state = load_state(&path)?;
            #[derive(serde::Serialize)]
            struct JobStatus {
                job: String,
                last_success: Option<DateTime<Utc>>,
                run_id: Option<String>,
            }
            let statuses: Vec<_> = state
                .jobs
                .values()
                .map(|job| {
                    let found = state
                        .receipts
                        .iter()
                        .filter(|r| {
                            r.payload.job == job.name
                                && r.payload.event == Event::Finish
                                && r.payload.status == Some(FinishStatus::Success)
                        })
                        .max_by_key(|r| r.payload.occurred_at);
                    JobStatus {
                        job: job.name.clone(),
                        last_success: found.map(|r| r.payload.occurred_at),
                        run_id: found.map(|r| r.payload.run_id.clone()),
                    }
                })
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&statuses)?);
            } else if statuses.is_empty() {
                println!("No jobs configured. Add one with `srr job add`.");
            } else {
                for item in statuses {
                    println!(
                        "{}  {}  {}",
                        item.job,
                        item.last_success
                            .map(|v| v.to_rfc3339())
                            .unwrap_or_else(|| "never".into()),
                        item.run_id.unwrap_or_else(|| "—".into())
                    );
                }
            }
        }
        Command::Export { week, output } => {
            let state = load_state(&path)?;
            let now = Utc::now();
            let (start, end) = week_bounds(week.as_deref(), now)?;
            let report = check(&state, start, std::cmp::min(end, now))?;
            let html = export_html(&report, start, end)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output, html)?;
            println!(
                "Wrote {} weekly evidence slots to {}",
                report.slots.len(),
                output.display()
            );
        }
    }
    Ok(0)
}
