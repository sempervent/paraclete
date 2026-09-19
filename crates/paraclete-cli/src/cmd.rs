//! Dispatch CLI commands to the HTTP client.

use std::time::{Duration, Instant};

use camino::Utf8PathBuf;
use paraclete_types::{AuthRole, JobStatus, ScanOptions, ScanProfile, ScanTarget};
use tokio::time::sleep;

use crate::cli::{Cli, Command, JobCmd, RunCmd, ScanCmd, SubmitArgs, TokenCmd};
use crate::client::ApiClient;
use crate::error::CliError;
use crate::render::{
    print_assets, print_findings, print_health, print_job_list, print_job_submission,
    print_job_view, print_json, print_run_list, print_run_summary, print_token_create,
    print_token_list, print_token_one, print_token_rotate,
};
use crate::wire::StartScanRequest;

pub async fn run(cli: Cli) -> Result<(), CliError> {
    let client = ApiClient::new(&cli.base_url, cli.token.clone())?;

    match cli.command {
        Command::Health => {
            let h = client.health().await?;
            print_health(cli.json, &h.status)?;
        }
        Command::WhoAmI => {
            let w = client.whoami().await?;
            if cli.json {
                print_json(&w)?;
            } else {
                println!("token_id: {}", w.token_id);
                println!("label:    {}", w.label);
                println!("role:     {:?}", w.role);
            }
        }
        Command::Diff { left_run_id, right_run_id } => {
            let d = client.diff_runs(left_run_id, right_run_id).await?;
            if cli.json {
                let s = serde_json::to_string(&d).map_err(|e| CliError::Decode(e.to_string()))?;
                println!("{s}");
            } else {
                print_json(&d)?;
            }
        }
        Command::Scan(ScanCmd::Submit(args)) => {
            let req = build_start_scan_request(args)?;
            let r = client.submit_scan_job(&req).await?;
            print_job_submission(cli.json, &r)?;
        }
        Command::Job(JobCmd::Get { job_id }) => {
            let j = client.get_job(job_id).await?;
            print_job_view(cli.json, &j)?;
        }
        Command::Job(JobCmd::Wait { job_id, interval_ms, timeout_secs }) => {
            job_wait(&client, cli.json, job_id, interval_ms, timeout_secs).await?;
        }
        Command::Job(JobCmd::List { status, limit, offset }) => {
            let r = client.list_jobs(status.as_deref(), limit, offset).await?;
            print_job_list(cli.json, &r)?;
        }
        Command::Run(RunCmd::Get { run_id }) => {
            let r = client.get_run_summary(run_id).await?;
            print_run_summary(cli.json, &r)?;
        }
        Command::Run(RunCmd::Report { run_id, output }) => {
            let v = client.get_run_report_json(run_id).await?;
            let s =
                serde_json::to_string_pretty(&v).map_err(|e| CliError::Decode(e.to_string()))?;
            if let Some(path) = output {
                std::fs::write(&path, s)?;
            } else {
                println!("{s}");
            }
        }
        Command::Run(RunCmd::Assets { run_id, limit, offset, inspection_status }) => {
            let p = client.list_assets(run_id, limit, offset, inspection_status.as_deref()).await?;
            print_assets(cli.json, &p)?;
        }
        Command::Run(RunCmd::Findings { run_id, limit, offset, severity, code }) => {
            let p = client
                .list_findings(run_id, limit, offset, severity.as_deref(), code.as_deref())
                .await?;
            print_findings(cli.json, &p)?;
        }
        Command::Run(RunCmd::List { target_kind, normalized_key, limit }) => {
            let items = client.list_runs_for_target(&target_kind, &normalized_key, limit).await?;
            print_run_list(cli.json, &items)?;
        }
        Command::Token(TokenCmd::Create { label, role, note }) => {
            let role: AuthRole =
                role.parse().map_err(|e: String| CliError::Msg(format!("invalid role: {e}")))?;
            let n = note.as_deref().map(str::trim).filter(|s| !s.is_empty());
            let r = client.admin_create_token(&label, role, n).await?;
            print_token_create(cli.json, &r)?;
        }
        Command::Token(TokenCmd::List) => {
            let r = client.admin_list_tokens().await?;
            print_token_list(cli.json, &r)?;
        }
        Command::Token(TokenCmd::Get { token_id }) => {
            let r = client.admin_get_token(token_id).await?;
            print_token_one(cli.json, &r)?;
        }
        Command::Token(TokenCmd::Disable { token_id }) => {
            let r = client.admin_disable_token(token_id).await?;
            print_token_one(cli.json, &r)?;
        }
        Command::Token(TokenCmd::Rotate { token_id }) => {
            let r = client.admin_rotate_token(token_id).await?;
            print_token_rotate(cli.json, &r)?;
        }
    }
    Ok(())
}

async fn job_wait(
    client: &ApiClient,
    json: bool,
    job_id: uuid::Uuid,
    interval_ms: u64,
    timeout_secs: Option<u64>,
) -> Result<(), CliError> {
    let start = Instant::now();
    loop {
        if let Some(t) = timeout_secs {
            if start.elapsed() > Duration::from_secs(t) {
                return Err(CliError::Msg(format!("timeout after {t}s waiting for job {job_id}")));
            }
        }
        let j = client.get_job(job_id).await?;
        match j.status {
            JobStatus::Succeeded => {
                let rid =
                    j.run_id.ok_or_else(|| CliError::Msg("succeeded job missing run_id".into()))?;
                if json {
                    print_json(&j)?;
                } else {
                    println!("succeeded run_id={rid}");
                }
                return Ok(());
            }
            JobStatus::Failed => {
                if json {
                    print_json(&j)?;
                } else if let Some(f) = &j.failure {
                    eprintln!("job failed: {:?} — {}", f.code, f.message);
                } else {
                    eprintln!("job failed");
                }
                return Err(CliError::Msg(format!("job {job_id} failed")));
            }
            JobStatus::Canceled => {
                if json {
                    print_json(&j)?;
                } else {
                    eprintln!("job canceled");
                }
                return Err(CliError::Msg(format!("job {job_id} canceled")));
            }
            JobStatus::Queued | JobStatus::Running => {
                sleep(Duration::from_millis(interval_ms.max(50))).await;
            }
        }
    }
}

fn build_start_scan_request(args: SubmitArgs) -> Result<StartScanRequest, CliError> {
    let target = build_target(args.file, args.directory, args.target_json)?;
    let profile = parse_profile(&args.profile)?;
    let options = ScanOptions {
        mode: paraclete_types::ScanMode::Full,
        max_files: args.max_files,
        format_hints: vec![],
    };
    Ok(StartScanRequest { target, profile, options, scan_id: None, redaction: None })
}

fn build_target(
    file: Option<std::path::PathBuf>,
    directory: Option<std::path::PathBuf>,
    target_json: Option<std::path::PathBuf>,
) -> Result<ScanTarget, CliError> {
    match (file, directory, target_json) {
        (Some(p), None, None) => {
            let path = Utf8PathBuf::from_path_buf(p)
                .map_err(|_| CliError::Msg("--file path must be valid UTF-8".into()))?;
            Ok(ScanTarget::LocalFile { path })
        }
        (None, Some(p), None) => {
            let path = Utf8PathBuf::from_path_buf(p)
                .map_err(|_| CliError::Msg("--directory path must be valid UTF-8".into()))?;
            Ok(ScanTarget::LocalDirectory { path })
        }
        (None, None, Some(path)) => {
            let s = std::fs::read_to_string(&path)?;
            let t: ScanTarget =
                serde_json::from_str(&s).map_err(|e| CliError::Msg(format!("target JSON: {e}")))?;
            Ok(t)
        }
        _ => Err(CliError::Msg(
            "exactly one of --file, --directory, or --target-json is required".into(),
        )),
    }
}

fn parse_profile(s: &str) -> Result<ScanProfile, CliError> {
    match s.to_lowercase().as_str() {
        "quick" => Ok(ScanProfile::Quick),
        "standard" => Ok(ScanProfile::Standard),
        "deep" => Ok(ScanProfile::Deep),
        "baseline" => Ok(ScanProfile::Baseline),
        _ => Err(CliError::Msg(format!("unknown profile `{s}`"))),
    }
}
