//! Application state, screens, and event application (testable without a terminal).

use std::sync::Arc;

use camino::Utf8PathBuf;
use paraclete_cli::api::{
    AuthTokenListResponse, AuthTokenSummaryView, HealthResponse, PagedAssets, PagedFindings,
    ScanJobListResponse, ScanJobView, WhoAmIResponse,
};
use paraclete_cli::wire::StartScanRequest;
use paraclete_cli::ApiClient;
use paraclete_types::{RunDiff, ScanMode, ScanOptions, ScanProfile, ScanRunListItem, ScanTarget};
use uuid::Uuid;

/// Main navigation surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Connect,
    Home,
    Jobs,
    JobWatch,
    ScanSubmit,
    RunsForTarget,
    RunSummary,
    Projections,
    Diff,
    Tokens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectField {
    BaseUrl,
    Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionTab {
    Assets,
    Findings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunsField {
    Kind,
    Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffField {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenField {
    Label,
    Role,
    DisableId,
}

/// Async results applied on the main loop (single writer to [`App`]).
#[derive(Debug)]
pub enum AppEvent {
    ConnectDone(Result<(HealthResponse, WhoAmIResponse), String>),
    JobsLoaded(Result<ScanJobListResponse, String>),
    JobPolled(Result<ScanJobView, String>),
    RunLoaded(Result<paraclete_cli::api::RunSummaryView, String>),
    RunsTargetLoaded(Result<Vec<ScanRunListItem>, String>),
    AssetsLoaded(Result<PagedAssets, String>),
    FindingsLoaded(Result<PagedFindings, String>),
    DiffLoaded(Result<RunDiff, String>),
    TokensLoaded(Result<AuthTokenListResponse, String>),
    TokenCreated(Result<paraclete_cli::api::AuthTokenCreateResponse, String>),
    TokenDisabled(Result<AuthTokenSummaryView, String>),
    ScanSubmitted(Result<paraclete_cli::api::ScanJobSubmissionResponse, String>),
}

pub struct App {
    pub screen: Screen,
    pub status_line: String,
    pub error_line: Option<String>,

    pub base_url: String,
    pub token: String,
    pub connect_focus: ConnectField,

    pub health: Option<HealthResponse>,
    pub whoami: Option<WhoAmIResponse>,

    pub jobs: Option<ScanJobListResponse>,
    pub jobs_status_filter: String,

    pub watch_job_id: Option<Uuid>,
    pub watch_job_id_buf: String,
    pub watch_job: Option<ScanJobView>,
    pub watch_poll_ms: u64,

    pub submit_path: String,
    pub submit_profile: String,
    pub submit_max_files: String,
    pub submit_file_not_dir: bool,
    pub submit_focus: SubmitFocus,

    pub runs_target_kind: String,
    pub runs_normalized_key: String,
    pub runs_field: RunsField,
    pub runs_list: Option<Vec<ScanRunListItem>>,

    pub run_id_input: String,
    pub selected_run_id: Option<Uuid>,
    pub run_summary: Option<paraclete_cli::api::RunSummaryView>,

    pub proj_tab: ProjectionTab,
    pub proj_limit: u32,
    pub proj_offset: u32,
    pub proj_assets: Option<PagedAssets>,
    pub proj_findings: Option<PagedFindings>,
    pub proj_inspection_filter: String,
    pub proj_severity_filter: String,

    pub diff_left: String,
    pub diff_right: String,
    pub diff_focus: DiffField,
    pub diff_result: Option<RunDiff>,

    pub tokens: Option<AuthTokenListResponse>,
    pub token_field: TokenField,
    pub token_new_label: String,
    pub token_new_role: String,
    pub token_disable_input: String,
    pub token_create_result: Option<String>,

    pub tick_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitFocus {
    Path,
    Profile,
    MaxFiles,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Connect,
            status_line: "Paraclete TUI — HTTP only. Enter base URL and token.".into(),
            error_line: None,
            base_url: std::env::var("PARACLETE_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".into()),
            token: std::env::var("PARACLETE_TOKEN").unwrap_or_default(),
            connect_focus: ConnectField::BaseUrl,
            health: None,
            whoami: None,
            jobs: None,
            jobs_status_filter: String::new(),
            watch_job_id: None,
            watch_job_id_buf: String::new(),
            watch_job: None,
            watch_poll_ms: 1000,
            submit_path: String::new(),
            submit_profile: "standard".into(),
            submit_max_files: "100000".into(),
            submit_file_not_dir: true,
            submit_focus: SubmitFocus::Path,
            runs_target_kind: "local_file".into(),
            runs_normalized_key: String::new(),
            runs_field: RunsField::Kind,
            runs_list: None,
            run_id_input: String::new(),
            selected_run_id: None,
            run_summary: None,
            proj_tab: ProjectionTab::Assets,
            proj_limit: 50,
            proj_offset: 0,
            proj_assets: None,
            proj_findings: None,
            proj_inspection_filter: String::new(),
            proj_severity_filter: String::new(),
            diff_left: String::new(),
            diff_right: String::new(),
            diff_focus: DiffField::Left,
            diff_result: None,
            tokens: None,
            token_field: TokenField::Label,
            token_new_label: String::new(),
            token_new_role: "operator".into(),
            token_disable_input: String::new(),
            token_create_result: None,
            tick_count: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build client from current base URL + token (no local store/engine).
    pub fn client(&self) -> Result<ApiClient, paraclete_cli::CliError> {
        let tok =
            if self.token.trim().is_empty() { None } else { Some(self.token.trim().to_string()) };
        ApiClient::new(self.base_url.trim(), tok)
    }

    pub fn clear_error(&mut self) {
        self.error_line = None;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_line = Some(msg.into());
    }

    pub fn apply_event(&mut self, ev: AppEvent) {
        match ev {
            AppEvent::ConnectDone(Ok((h, w))) => {
                self.health = Some(h);
                self.whoami = Some(w);
                self.screen = Screen::Home;
                self.status_line = "Connected.".into();
                self.clear_error();
            }
            AppEvent::ConnectDone(Err(e)) => {
                self.set_error(e);
            }
            AppEvent::JobsLoaded(Ok(j)) => {
                self.jobs = Some(j);
                self.clear_error();
            }
            AppEvent::JobsLoaded(Err(e)) => self.set_error(e),
            AppEvent::JobPolled(Ok(j)) => {
                self.watch_job = Some(j);
                self.clear_error();
            }
            AppEvent::JobPolled(Err(e)) => self.set_error(e),
            AppEvent::RunLoaded(Ok(r)) => {
                self.run_summary = Some(r);
                self.clear_error();
            }
            AppEvent::RunLoaded(Err(e)) => self.set_error(e),
            AppEvent::RunsTargetLoaded(Ok(v)) => {
                self.runs_list = Some(v);
                self.clear_error();
            }
            AppEvent::RunsTargetLoaded(Err(e)) => self.set_error(e),
            AppEvent::AssetsLoaded(Ok(a)) => {
                self.proj_assets = Some(a);
                self.clear_error();
            }
            AppEvent::AssetsLoaded(Err(e)) => self.set_error(e),
            AppEvent::FindingsLoaded(Ok(f)) => {
                self.proj_findings = Some(f);
                self.clear_error();
            }
            AppEvent::FindingsLoaded(Err(e)) => self.set_error(e),
            AppEvent::DiffLoaded(Ok(d)) => {
                self.diff_result = Some(d);
                self.clear_error();
            }
            AppEvent::DiffLoaded(Err(e)) => self.set_error(e),
            AppEvent::TokensLoaded(Ok(t)) => {
                self.tokens = Some(t);
                self.clear_error();
            }
            AppEvent::TokensLoaded(Err(e)) => self.set_error(e),
            AppEvent::TokenCreated(Ok(r)) => {
                self.token_create_result = Some(format!(
                    "Created token {} — secret (save now): {}",
                    r.token_id, r.token_secret
                ));
                self.clear_error();
            }
            AppEvent::TokenCreated(Err(e)) => self.set_error(e),
            AppEvent::TokenDisabled(Ok(_)) => {
                self.status_line = "Token disabled.".into();
                self.clear_error();
            }
            AppEvent::TokenDisabled(Err(e)) => self.set_error(e),
            AppEvent::ScanSubmitted(Ok(s)) => {
                self.watch_job_id = Some(s.job_id);
                self.watch_job_id_buf = s.job_id.to_string();
                self.screen = Screen::JobWatch;
                self.status_line = format!("Job queued: {}", s.job_id);
                self.clear_error();
            }
            AppEvent::ScanSubmitted(Err(e)) => self.set_error(e),
        }
    }

    /// Whether [`Screen::JobWatch`] should trigger a poll this tick.
    pub fn should_poll_job(&self) -> bool {
        matches!(self.screen, Screen::JobWatch)
            && self.watch_job_id.is_some()
            && self.watch_poll_ms > 0
    }

    pub fn terminal_job_reached(&self) -> bool {
        self.watch_job.as_ref().is_some_and(|j| {
            use paraclete_types::JobStatus;
            matches!(j.status, JobStatus::Succeeded | JobStatus::Failed | JobStatus::Canceled)
        })
    }

    pub fn watch_job_id_str(&self) -> &str {
        &self.watch_job_id_buf
    }

    pub fn watch_job_id_str_push(&mut self, c: char) {
        self.watch_job_id_buf.push(c);
    }

    pub fn watch_job_id_str_pop(&mut self) {
        self.watch_job_id_buf.pop();
    }

    pub fn focus_runs_kind(&mut self) {
        self.runs_field = RunsField::Kind;
    }

    pub fn focus_runs_key(&mut self) {
        self.runs_field = RunsField::Key;
    }

    pub fn runs_type_key(&mut self, c: char) {
        match self.runs_field {
            RunsField::Kind => self.runs_target_kind.push(c),
            RunsField::Key => self.runs_normalized_key.push(c),
        }
    }

    pub fn runs_backspace(&mut self) {
        match self.runs_field {
            RunsField::Kind => {
                self.runs_target_kind.pop();
            }
            RunsField::Key => {
                self.runs_normalized_key.pop();
            }
        }
    }

    pub fn diff_focus_left(&mut self) {
        self.diff_focus = DiffField::Left;
    }

    pub fn diff_focus_right(&mut self) {
        self.diff_focus = DiffField::Right;
    }

    pub fn diff_type(&mut self, c: char) {
        match self.diff_focus {
            DiffField::Left => self.diff_left.push(c),
            DiffField::Right => self.diff_right.push(c),
        }
    }

    pub fn diff_backspace(&mut self) {
        match self.diff_focus {
            DiffField::Left => {
                self.diff_left.pop();
            }
            DiffField::Right => {
                self.diff_right.pop();
            }
        }
    }

    pub fn token_focus_cycle(&mut self) {
        self.token_field = match self.token_field {
            TokenField::Label => TokenField::Role,
            TokenField::Role => TokenField::DisableId,
            TokenField::DisableId => TokenField::Label,
        };
    }

    pub fn token_type(&mut self, ch: char) {
        match self.token_field {
            TokenField::Label => self.token_new_label.push(ch),
            TokenField::Role => self.token_new_role.push(ch),
            TokenField::DisableId => self.token_disable_input.push(ch),
        }
    }

    pub fn token_backspace(&mut self) {
        match self.token_field {
            TokenField::Label => {
                self.token_new_label.pop();
            }
            TokenField::Role => {
                self.token_new_role.pop();
            }
            TokenField::DisableId => {
                self.token_disable_input.pop();
            }
        }
    }

    pub fn cycle_submit_profile(&mut self) {
        self.submit_profile = match self.submit_profile.as_str() {
            "quick" => "standard".into(),
            "standard" => "deep".into(),
            "deep" => "baseline".into(),
            _ => "quick".into(),
        };
    }

    pub fn submit_type_char(&mut self, c: char) {
        match self.submit_focus {
            SubmitFocus::Path => self.submit_path.push(c),
            SubmitFocus::Profile => self.submit_profile.push(c),
            SubmitFocus::MaxFiles => self.submit_max_files.push(c),
        }
    }

    pub fn submit_backspace(&mut self) {
        match self.submit_focus {
            SubmitFocus::Path => {
                self.submit_path.pop();
            }
            SubmitFocus::Profile => {
                self.submit_profile.pop();
            }
            SubmitFocus::MaxFiles => {
                self.submit_max_files.pop();
            }
        }
    }

    pub fn submit_tab_focus(&mut self) {
        self.submit_focus = match self.submit_focus {
            SubmitFocus::Path => SubmitFocus::Profile,
            SubmitFocus::Profile => SubmitFocus::MaxFiles,
            SubmitFocus::MaxFiles => SubmitFocus::Path,
        };
    }
}

pub fn parse_profile(s: &str) -> Result<ScanProfile, String> {
    match s.trim().to_lowercase().as_str() {
        "quick" => Ok(ScanProfile::Quick),
        "standard" => Ok(ScanProfile::Standard),
        "deep" => Ok(ScanProfile::Deep),
        "baseline" => Ok(ScanProfile::Baseline),
        _ => Err(format!("unknown profile `{s}` (quick|standard|deep|baseline)")),
    }
}

pub fn build_start_scan(
    path: &str,
    file: bool,
    profile: ScanProfile,
    max_files: u64,
) -> Result<StartScanRequest, String> {
    let p = Utf8PathBuf::from(path.trim());
    let target = if file {
        ScanTarget::LocalFile { path: p }
    } else {
        ScanTarget::LocalDirectory { path: p }
    };
    Ok(StartScanRequest {
        target,
        profile,
        options: ScanOptions { mode: ScanMode::Full, max_files, format_hints: vec![] },
        scan_id: None,
        redaction: None,
    })
}

/// Spawn connect (health + whoami) on the runtime; sender receives [`AppEvent`].
pub fn spawn_connect(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), {
                let t = token.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .map_err(|e| e.to_string())?;
            let h = c.health().await.map_err(|e| e.to_string())?;
            let w = c.whoami().await.map_err(|e| e.to_string())?;
            Ok((h, w))
        }
        .await;
        let _ = tx.send(AppEvent::ConnectDone(res));
    });
}

pub fn spawn_load_jobs(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    let st = app.jobs_status_filter.clone();
    let limit = 20u32;
    let offset = 0u32;
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            let sf = if st.trim().is_empty() { None } else { Some(st.trim()) };
            c.list_jobs(sf, limit, offset).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::JobsLoaded(res));
    });
}

pub fn spawn_poll_job(
    tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>,
    app: &App,
    job_id: Uuid,
) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.get_job(job_id).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::JobPolled(res));
    });
}

pub fn spawn_submit_scan(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    let path = app.submit_path.clone();
    let file = app.submit_file_not_dir;
    let prof = app.submit_profile.clone();
    let max: u64 = app.submit_max_files.parse().unwrap_or(100_000);
    tokio::spawn(async move {
        let res = async {
            let profile = parse_profile(&prof)?;
            let body = build_start_scan(&path, file, profile, max)?;
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.submit_scan_job(&body).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::ScanSubmitted(res));
    });
}

pub fn spawn_run_summary(
    tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>,
    app: &App,
    run_id: Uuid,
) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.get_run_summary(run_id).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::RunLoaded(res));
    });
}

pub fn spawn_runs_for_target(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    let kind = app.runs_target_kind.clone();
    let key = app.runs_normalized_key.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.list_runs_for_target(&kind, &key, 50).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::RunsTargetLoaded(res));
    });
}

pub fn spawn_load_assets(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let Some(rid) = app.selected_run_id else {
        return;
    };
    let base = app.base_url.clone();
    let token = app.token.clone();
    let limit = app.proj_limit;
    let offset = app.proj_offset;
    let ins = app.proj_inspection_filter.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            let f = if ins.trim().is_empty() { None } else { Some(ins.trim()) };
            c.list_assets(rid, limit, offset, f).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::AssetsLoaded(res));
    });
}

pub fn spawn_load_findings(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let Some(rid) = app.selected_run_id else {
        return;
    };
    let base = app.base_url.clone();
    let token = app.token.clone();
    let limit = app.proj_limit;
    let offset = app.proj_offset;
    let sev = app.proj_severity_filter.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            let s = if sev.trim().is_empty() { None } else { Some(sev.trim()) };
            c.list_findings(rid, limit, offset, s, None).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::FindingsLoaded(res));
    });
}

pub fn spawn_diff(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let left: Result<Uuid, String> =
        app.diff_left.trim().parse().map_err(|_| "invalid left UUID".into());
    let right: Result<Uuid, String> =
        app.diff_right.trim().parse().map_err(|_| "invalid right UUID".into());
    let Ok(left) = left else {
        let _ = tx.send(AppEvent::DiffLoaded(Err("invalid left UUID".into())));
        return;
    };
    let Ok(right) = right else {
        let _ = tx.send(AppEvent::DiffLoaded(Err("invalid right UUID".into())));
        return;
    };
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.diff_runs(left, right).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::DiffLoaded(res));
    });
}

pub fn spawn_tokens(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.admin_list_tokens().await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::TokensLoaded(res));
    });
}

pub fn spawn_token_create(tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>, app: &App) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    let label = app.token_new_label.clone();
    let role_s = app.token_new_role.clone();
    tokio::spawn(async move {
        let res = async {
            let role = role_s.parse::<paraclete_types::AuthRole>().map_err(|e| e.to_string())?;
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.admin_create_token(&label, role, None).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::TokenCreated(res));
    });
}

pub fn spawn_token_disable(
    tx: Arc<tokio::sync::mpsc::UnboundedSender<AppEvent>>,
    app: &App,
    token_id: Uuid,
) {
    let base = app.base_url.clone();
    let token = app.token.clone();
    tokio::spawn(async move {
        let res = async {
            let c = ApiClient::new(base.trim(), Some(token)).map_err(|e| e.to_string())?;
            c.admin_disable_token(token_id).await.map_err(|e| e.to_string())
        }
        .await;
        let _ = tx.send(AppEvent::TokenDisabled(res));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_at_connect() {
        let a = App::new();
        assert_eq!(a.screen, Screen::Connect);
    }

    #[test]
    fn connect_success_moves_home() {
        let mut a = App::new();
        a.apply_event(AppEvent::ConnectDone(Ok((
            paraclete_cli::api::HealthResponse { status: "ok".into() },
            paraclete_cli::api::WhoAmIResponse {
                token_id: Uuid::nil(),
                label: "t".into(),
                role: paraclete_types::AuthRole::Operator,
            },
        ))));
        assert_eq!(a.screen, Screen::Home);
    }

    #[test]
    fn parse_profile_accepts_standard() {
        assert!(parse_profile("standard").is_ok());
    }

    #[test]
    fn submit_focus_cycles() {
        let mut a = App::new();
        assert_eq!(a.submit_focus, SubmitFocus::Path);
        a.submit_tab_focus();
        assert_eq!(a.submit_focus, SubmitFocus::Profile);
    }
}
