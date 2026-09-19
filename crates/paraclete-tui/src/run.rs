//! Crossterm + ratatui loop; forwards input to [`crate::App`] and draws.

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use paraclete_types::JobStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app::{
    spawn_connect, spawn_diff, spawn_load_assets, spawn_load_findings, spawn_load_jobs,
    spawn_poll_job, spawn_run_summary, spawn_runs_for_target, spawn_submit_scan,
    spawn_token_create, spawn_token_disable, spawn_tokens, App, AppEvent, ConnectField,
    ProjectionTab, Screen, SubmitFocus,
};
use crate::RunError;

const TICK_MS: u64 = 250;

pub async fn run_async() -> Result<(), RunError> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (evt_tx, mut evt_rx) = mpsc::unbounded_channel::<AppEvent>();
    let evt_tx = std::sync::Arc::new(evt_tx);

    let (key_tx, mut key_rx) = mpsc::unbounded_channel::<event::Event>();
    std::thread::spawn(move || loop {
        if let Ok(e) = event::read() {
            let _ = key_tx.send(e);
        }
    });

    let mut app = App::new();
    let mut tick = tokio::time::interval(Duration::from_millis(TICK_MS));

    loop {
        terminal.draw(|f| draw(f, &app))?;

        tokio::select! {
            Some(ev) = evt_rx.recv() => {
                app.apply_event(ev);
            }
            Some(key_ev) = key_rx.recv() => {
                if let Event::Key(key) = key_ev {
                    if key.kind == KeyEventKind::Press && handle_key(&mut app, &evt_tx, key.code) {
                        break;
                    }
                }
            }
            _ = tick.tick() => {
                app.tick_count = app.tick_count.wrapping_add(1);
                on_tick(&mut app, &evt_tx);
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn on_tick(app: &mut App, tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>) {
    if !app.should_poll_job() {
        return;
    }
    let Some(jid) = app.watch_job_id else {
        return;
    };
    let every = (app.watch_poll_ms / TICK_MS).max(1);
    if app.tick_count % every != 0 {
        return;
    }
    spawn_poll_job(tx.clone(), app, jid);
}

fn handle_key(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    app.clear_error();

    match app.screen {
        Screen::Connect => handle_connect(app, tx, code),
        Screen::Home => handle_home(app, tx, code),
        Screen::Jobs => handle_jobs(app, tx, code),
        Screen::JobWatch => handle_job_watch(app, tx, code),
        Screen::ScanSubmit => handle_submit(app, tx, code),
        Screen::RunsForTarget => handle_runs_target(app, tx, code),
        Screen::RunSummary => handle_run_summary(app, tx, code),
        Screen::Projections => handle_projections(app, tx, code),
        Screen::Diff => handle_diff(app, tx, code),
        Screen::Tokens => handle_tokens(app, tx, code),
    }
}

fn handle_connect(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Tab => {
            app.connect_focus = match app.connect_focus {
                ConnectField::BaseUrl => ConnectField::Token,
                ConnectField::Token => ConnectField::BaseUrl,
            };
        }
        KeyCode::Char('q') => return true,
        KeyCode::Enter => {
            spawn_connect(tx.clone(), app);
        }
        KeyCode::Char(c) => match app.connect_focus {
            ConnectField::BaseUrl => app.base_url.push(c),
            ConnectField::Token => app.token.push(c),
        },
        KeyCode::Backspace => match app.connect_focus {
            ConnectField::BaseUrl => {
                app.base_url.pop();
            }
            ConnectField::Token => {
                app.token.pop();
            }
        },
        _ => {}
    }
    false
}

fn handle_home(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('1') => {
            app.screen = Screen::Jobs;
            spawn_load_jobs(tx.clone(), app);
        }
        KeyCode::Char('2') => app.screen = Screen::ScanSubmit,
        KeyCode::Char('3') => app.screen = Screen::RunsForTarget,
        KeyCode::Char('4') => app.screen = Screen::RunSummary,
        KeyCode::Char('5') => {
            if app.selected_run_id.is_some() {
                app.screen = Screen::Projections;
                spawn_load_assets(tx.clone(), app);
                spawn_load_findings(tx.clone(), app);
            } else {
                app.set_error("Set a run id on Run summary (4) first.");
            }
        }
        KeyCode::Char('6') => app.screen = Screen::Diff,
        KeyCode::Char('7') => {
            app.screen = Screen::Tokens;
            spawn_tokens(tx.clone(), app);
        }
        KeyCode::Char('r') => {
            spawn_connect(tx.clone(), app);
        }
        KeyCode::Esc => {}
        _ => {}
    }
    false
}

fn handle_jobs(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Char('r') => spawn_load_jobs(tx.clone(), app),
        KeyCode::Char('w') => app.screen = Screen::JobWatch,
        _ => {}
    }
    false
}

fn handle_job_watch(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Char('l') => {
            if let Ok(id) = app.watch_job_id_str().parse::<Uuid>() {
                app.watch_job_id = Some(id);
                spawn_poll_job(tx.clone(), app, id);
            } else {
                app.set_error("Invalid job UUID in buffer");
            }
        }
        KeyCode::Char(c) if c.is_ascii_hexdigit() || c == '-' => {
            app.watch_job_id_str_push(c);
        }
        KeyCode::Backspace => {
            app.watch_job_id_str_pop();
        }
        _ => {}
    }
    false
}

fn handle_submit(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Char('t') => app.submit_file_not_dir = !app.submit_file_not_dir,
        KeyCode::Tab => app.submit_tab_focus(),
        KeyCode::Char('y') => app.cycle_submit_profile(),
        KeyCode::Enter => spawn_submit_scan(tx.clone(), app),
        KeyCode::Char(c) => app.submit_type_char(c),
        KeyCode::Backspace => app.submit_backspace(),
        _ => {}
    }
    false
}

fn handle_runs_target(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Enter => spawn_runs_for_target(tx.clone(), app),
        KeyCode::Char('k') => app.focus_runs_kind(),
        KeyCode::Char('n') => app.focus_runs_key(),
        KeyCode::Char(c) => app.runs_type_key(c),
        KeyCode::Backspace => app.runs_backspace(),
        _ => {}
    }
    false
}

fn handle_run_summary(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Enter | KeyCode::Char('l') => {
            if let Ok(id) = app.run_id_input.parse::<Uuid>() {
                app.selected_run_id = Some(id);
                spawn_run_summary(tx.clone(), app, id);
            } else {
                app.set_error("Invalid run UUID");
            }
        }
        KeyCode::Char(c) if c.is_ascii_hexdigit() || c == '-' => app.run_id_input.push(c),
        KeyCode::Backspace => {
            app.run_id_input.pop();
        }
        _ => {}
    }
    false
}

fn handle_projections(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Char('1') => app.proj_tab = ProjectionTab::Assets,
        KeyCode::Char('2') => app.proj_tab = ProjectionTab::Findings,
        KeyCode::Char('n') => {
            app.proj_offset = app.proj_offset.saturating_add(app.proj_limit);
            match app.proj_tab {
                ProjectionTab::Assets => spawn_load_assets(tx.clone(), app),
                ProjectionTab::Findings => spawn_load_findings(tx.clone(), app),
            }
        }
        KeyCode::Char('p') => {
            app.proj_offset = app.proj_offset.saturating_sub(app.proj_limit);
            match app.proj_tab {
                ProjectionTab::Assets => spawn_load_assets(tx.clone(), app),
                ProjectionTab::Findings => spawn_load_findings(tx.clone(), app),
            }
        }
        KeyCode::Char('r') => match app.proj_tab {
            ProjectionTab::Assets => spawn_load_assets(tx.clone(), app),
            ProjectionTab::Findings => spawn_load_findings(tx.clone(), app),
        },
        _ => {}
    }
    false
}

fn handle_diff(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Enter => spawn_diff(tx.clone(), app),
        KeyCode::Char('[') => app.diff_focus_left(),
        KeyCode::Char(']') => app.diff_focus_right(),
        KeyCode::Char(c) => app.diff_type(c),
        KeyCode::Backspace => app.diff_backspace(),
        _ => {}
    }
    false
}

fn handle_tokens(
    app: &mut App,
    tx: &std::sync::Arc<mpsc::UnboundedSender<AppEvent>>,
    code: KeyCode,
) -> bool {
    match code {
        KeyCode::Esc | KeyCode::Char('b') => app.screen = Screen::Home,
        KeyCode::Char('r') => spawn_tokens(tx.clone(), app),
        KeyCode::Char('c') => spawn_token_create(tx.clone(), app),
        KeyCode::Char('d') => {
            if let Ok(id) = app.token_disable_input.parse::<Uuid>() {
                spawn_token_disable(tx.clone(), app, id);
            } else {
                app.set_error("Invalid token UUID for disable");
            }
        }
        KeyCode::Char('x') => app.token_focus_cycle(),
        KeyCode::Char(ch) => app.token_type(ch),
        KeyCode::Backspace => app.token_backspace(),
        _ => {}
    }
    false
}

// --- draw ---

fn draw(f: &mut Frame, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" paraclete-tui — HTTP API only ");
    let inner = block.inner(f.area());
    f.render_widget(block, f.area());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(inner);

    match app.screen {
        Screen::Connect => draw_connect(f, app, layout[0]),
        Screen::Home => draw_home(f, app, layout[0]),
        Screen::Jobs => draw_jobs(f, app, layout[0]),
        Screen::JobWatch => draw_job_watch(f, app, layout[0]),
        Screen::ScanSubmit => draw_submit(f, app, layout[0]),
        Screen::RunsForTarget => draw_runs(f, app, layout[0]),
        Screen::RunSummary => draw_run_summary(f, app, layout[0]),
        Screen::Projections => draw_proj(f, app, layout[0]),
        Screen::Diff => draw_diff(f, app, layout[0]),
        Screen::Tokens => draw_tokens(f, app, layout[0]),
    }

    let status = Paragraph::new(vec![
        Line::from(Span::styled(
            format!(" {} ", app.status_line),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(
            app.error_line
                .as_ref()
                .map(|e| Span::styled(format!(" Error: {e} "), Style::default().fg(Color::Red)))
                .unwrap_or_else(|| Span::raw("")),
        ),
    ])
    .block(Block::default().borders(Borders::ALL).title(" status "));
    f.render_widget(status, layout[1]);
}

fn draw_connect(f: &mut Frame, app: &App, area: Rect) {
    let focus = match app.connect_focus {
        ConnectField::BaseUrl => "base URL",
        ConnectField::Token => "token",
    };
    let text = vec![
        Line::from(
            "Connect to Paraclete (Tab switch field, Enter = test health + whoami, q = quit)",
        ),
        Line::from(""),
        Line::from(vec![
            Span::raw("Base URL "),
            Span::styled(
                format!("[{}]", if focus == "base URL" { "*" } else { " " }),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(app.base_url.as_str()),
        Line::from(""),
        Line::from(vec![
            Span::raw("Token "),
            Span::styled(
                format!("[{}]", if focus == "token" { "*" } else { " " }),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(app.token.as_str()),
    ];
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" connection "));
    f.render_widget(p, area);
}

fn draw_home(f: &mut Frame, app: &App, area: Rect) {
    let h = app.health.as_ref().map(|x| x.status.as_str()).unwrap_or("—");
    let who = app
        .whoami
        .as_ref()
        .map(|w| format!("{} / {:?}", w.label, w.role))
        .unwrap_or_else(|| "—".into());

    let menu = vec![
        ListItem::new("1 Jobs list   2 Submit scan   3 Runs for target"),
        ListItem::new("4 Run summary (enter run UUID)   5 Assets/findings (needs run)"),
        ListItem::new("6 Diff   7 Tokens (admin)   r Re-check health+whoami"),
        ListItem::new("q Quit"),
    ];
    let list = List::new(menu)
        .block(Block::default().title(" menu "))
        .highlight_style(Style::default().fg(Color::Yellow));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(4)])
        .split(area);

    let info = Paragraph::new(vec![
        Line::from(format!("Health: {h}")),
        Line::from(format!("Whoami: {who}")),
    ])
    .block(Block::default().title(" session "));
    f.render_widget(info, chunks[0]);
    f.render_widget(list, chunks[1]);
}

fn draw_jobs(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<ListItem> =
        vec![ListItem::new("Esc back — r refresh — w go to job watch (set id there)")];
    if let Some(j) = &app.jobs {
        for it in &j.items {
            lines.push(ListItem::new(format!(
                "{}  {:?}  {}  {}",
                it.job_id, it.status, it.target_kind, it.normalized_target_key
            )));
        }
        lines.push(ListItem::new(format!("total {}  limit {}", j.total, j.limit)));
    }
    let list = List::new(lines).block(Block::default().title(" jobs "));
    f.render_widget(list, area);
}

fn draw_job_watch(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);

    let id_line = Paragraph::new(format!(
        "Job UUID buffer: {}  (type hex uuid, l = load/poll)  Esc back",
        app.watch_job_id_str()
    ))
    .block(Block::default().title(" job id "));
    f.render_widget(id_line, chunks[0]);

    let body = if let Some(j) = &app.watch_job {
        let term =
            matches!(j.status, JobStatus::Succeeded | JobStatus::Failed | JobStatus::Canceled);
        let mut s = format!(
            "status: {:?}\nattempts: {}\nrun_id: {:?}\nworker: {:?}\nlease: {:?}\nrecovery: {:?}\n",
            j.status, j.attempt_count, j.run_id, j.worker_id, j.leased_until, j.recovery_note
        );
        if let Some(f) = &j.failure {
            s.push_str(&format!("failure: {:?} {}\n", f.code, f.message));
        }
        if term {
            s.push_str("\nTerminal state reached.");
        }
        Paragraph::new(s)
    } else {
        Paragraph::new("No poll yet.")
    };
    f.render_widget(body.block(Block::default().title(" job ")), chunks[1]);
}

fn draw_submit(f: &mut Frame, app: &App, area: Rect) {
    let mode = if app.submit_file_not_dir { "file" } else { "directory" };
    let focus = match app.submit_focus {
        SubmitFocus::Path => "path",
        SubmitFocus::Profile => "profile",
        SubmitFocus::MaxFiles => "max_files",
    };
    let p = Paragraph::new(vec![
        Line::from(format!(
            "Tab = cycle field ({focus}) | t = file/dir ({mode}) | y = cycle profile | Enter = submit | Esc back"
        )),
        Line::from(format!(
            "path: {}  |  profile: {}  |  max_files: {}",
            app.submit_path, app.submit_profile, app.submit_max_files
        )),
    ])
    .block(Block::default().title(" submit scan "));
    f.render_widget(p, area);
}

fn draw_runs(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from("k = edit kind, n = edit key, Enter = load, Esc back"),
        Line::from(format!("kind: {}", app.runs_target_kind)),
        Line::from(format!("key:  {}", app.runs_normalized_key)),
        Line::from(""),
    ];
    if let Some(r) = &app.runs_list {
        for x in r {
            lines.push(Line::from(format!(
                "{}  {:?}  {}",
                x.run_id, x.run_outcome, x.target_identity.normalized_key
            )));
        }
    }
    f.render_widget(Paragraph::new(lines).block(Block::default().title(" runs for target ")), area);
}

fn draw_run_summary(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);

    let head =
        Paragraph::new(format!("Run UUID: {}  |  l/Enter load  |  Esc back", app.run_id_input))
            .block(Block::default().title(" input "));
    f.render_widget(head, chunks[0]);

    let body = if let Some(r) = &app.run_summary {
        Paragraph::new(format!(
            "outcome: {:?}\nsha256: {}\nstarted: {}\ncompleted: {}\nsummary: {:?}",
            r.run_outcome, r.report_sha256, r.started_at, r.completed_at, r.summary
        ))
    } else {
        Paragraph::new("—")
    };
    f.render_widget(body.block(Block::default().title(" summary ")), chunks[1]);
}

fn draw_proj(f: &mut Frame, app: &App, area: Rect) {
    let tab = match app.proj_tab {
        ProjectionTab::Assets => "assets (1)",
        ProjectionTab::Findings => "findings (2)",
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(6)])
        .split(area);

    let head = Paragraph::new(format!(
        "run {:?} | tab {tab} | n/p page | r refresh | Esc home",
        app.selected_run_id
    ));
    f.render_widget(head, chunks[0]);

    let text = match app.proj_tab {
        ProjectionTab::Assets => app
            .proj_assets
            .as_ref()
            .map(|a| {
                a.items
                    .iter()
                    .map(|r| format!("{}  {}  {}", r.path, r.format, r.inspection_status))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_else(|| "—".into()),
        ProjectionTab::Findings => app
            .proj_findings
            .as_ref()
            .map(|x| {
                x.items
                    .iter()
                    .map(|r| format!("{}  {}  {}", r.code, r.severity, r.fingerprint))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_else(|| "—".into()),
    };
    f.render_widget(Paragraph::new(text).block(Block::default().title(" projections ")), chunks[1]);
}

fn draw_diff(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(4)])
        .split(area);

    let head = Paragraph::new(vec![
        Line::from(format!(
            "Left: {}  Right: {}  |  [ ] switch field  |  Enter run  |  Esc back",
            app.diff_left, app.diff_right
        )),
        Line::from(format!("focus: {:?}", app.diff_focus)),
    ]);
    f.render_widget(head, chunks[0]);

    let body = if let Some(d) = &app.diff_result {
        Paragraph::new(format!(
            "datasets +:{:?} -:{:?}\nfindings +{} -{}\nsummary delta: {:?}",
            d.datasets_added,
            d.datasets_removed,
            d.findings.added.len(),
            d.findings.removed.len(),
            d.summary_delta
        ))
    } else {
        Paragraph::new("—")
    };
    f.render_widget(body.block(Block::default().title(" diff ")), chunks[1]);
}

fn draw_tokens(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(4)])
        .split(area);

    let head = Paragraph::new(vec![
        Line::from(
            "r refresh | c create (label/role fields) | d disable UUID in buffer | x cycle field",
        ),
        Line::from(format!(
            "label={} role={} disable_buf={}",
            app.token_new_label, app.token_new_role, app.token_disable_input
        )),
        Line::from(app.token_create_result.as_deref().unwrap_or("")),
    ]);
    f.render_widget(head, chunks[0]);

    let body = if let Some(t) = &app.tokens {
        let s = t
            .items
            .iter()
            .map(|x| {
                let lu = x.last_used_at.map(|d| d.to_rfc3339()).unwrap_or_else(|| "-".into());
                format!("{}  {:?}  {:?}  {}  last={}", x.token_id, x.role, x.status, x.label, lu)
            })
            .collect::<Vec<_>>()
            .join("\n");
        Paragraph::new(s)
    } else {
        Paragraph::new("—")
    };
    f.render_widget(body.block(Block::default().title(" tokens ")), chunks[1]);
}
