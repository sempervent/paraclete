//! `clap` CLI definition (thin client over `/api/v1`).

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use uuid::Uuid;

const DEFAULT_BASE: &str = "http://127.0.0.1:8080";

#[derive(Parser, Debug)]
#[command(name = "paraclete", version, about = "Paraclete HTTP client — jobs, runs, and diffs.")]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "PARACLETE_BASE_URL",
        default_value = DEFAULT_BASE,
        help = "Paraclete HTTP API base URL"
    )]
    pub base_url: String,
    #[arg(long, global = true, help = "Emit JSON instead of human-readable output")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check `GET /api/v1/health`
    Health,
    /// Fetch diff between two runs
    Diff { left_run_id: Uuid, right_run_id: Uuid },
    #[command(subcommand)]
    Scan(ScanCmd),
    #[command(subcommand)]
    Job(JobCmd),
    #[command(subcommand)]
    Run(RunCmd),
}

#[derive(Subcommand, Debug)]
pub enum ScanCmd {
    /// Submit an async scan job (`POST /api/v1/jobs/scans`)
    Submit(SubmitArgs),
}

#[derive(clap::Args, Debug)]
pub struct SubmitArgs {
    /// Scan a single local file
    #[arg(long, group = "tgt")]
    pub file: Option<PathBuf>,
    /// Scan a local directory
    #[arg(long, group = "tgt")]
    pub directory: Option<PathBuf>,
    /// Path to JSON file containing a `ScanTarget` object
    #[arg(long, group = "tgt")]
    pub target_json: Option<PathBuf>,
    #[arg(short, long, default_value = "standard")]
    pub profile: String,
    #[arg(long, default_value_t = 100_000_u64)]
    pub max_files: u64,
}

#[derive(Subcommand, Debug)]
pub enum JobCmd {
    /// Fetch job status (`GET /api/v1/jobs/{id}`)
    Get { job_id: Uuid },
    /// Poll until terminal state (succeeded / failed / canceled)
    Wait {
        job_id: Uuid,
        #[arg(long, default_value_t = 500_u64, help = "Poll interval in milliseconds")]
        interval_ms: u64,
        #[arg(long, help = "Give up after this many seconds")]
        timeout_secs: Option<u64>,
    },
    /// List recent jobs
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 50_u32)]
        limit: u32,
        #[arg(long, default_value_t = 0_u32)]
        offset: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum RunCmd {
    /// Run summary (no full report blob)
    Get { run_id: Uuid },
    /// Full canonical report JSON
    Report {
        run_id: Uuid,
        #[arg(short, long, help = "Write report JSON to file instead of stdout")]
        output: Option<PathBuf>,
    },
    /// Paginated assets
    Assets {
        run_id: Uuid,
        #[arg(long, default_value_t = 50_u32)]
        limit: u32,
        #[arg(long, default_value_t = 0_u32)]
        offset: u32,
        #[arg(long)]
        inspection_status: Option<String>,
    },
    /// Paginated findings
    Findings {
        run_id: Uuid,
        #[arg(long, default_value_t = 50_u32)]
        limit: u32,
        #[arg(long, default_value_t = 0_u32)]
        offset: u32,
        #[arg(long)]
        severity: Option<String>,
        #[arg(long)]
        code: Option<String>,
    },
    /// List runs for a target identity
    List {
        #[arg(long)]
        target_kind: String,
        #[arg(long)]
        normalized_key: String,
        #[arg(long, default_value_t = 50_i64)]
        limit: i64,
    },
}
