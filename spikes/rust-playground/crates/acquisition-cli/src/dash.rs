//! `acq dash` — live TUI dashboard over the daemon's `dashboard` request.
//!
//! Pure client: polls the daemon a few times a second and renders what comes
//! back. All state lives daemon-side, so closing the dashboard changes
//! nothing, and several dashboards can watch the same daemon.

use std::io::IsTerminal as _;
use std::time::Duration;

use acquisition_core::job::{JobInfo, JobState};
use acquisition_core::protocol::{ErrorRecord, Request, Response};
use acquisition_core::ratelimit::{DegradedEndpoint, PolicyStatus, SendRecord};
use anyhow::{Result, bail};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize as _};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};

use crate::client::Client;

const POLL: Duration = Duration::from_millis(250);
/// Rows of policy detail visible at once when a policy is expanded; longer
/// detail scrolls with ↑/↓.
const DETAIL_ROWS: u16 = 12;

/// UI-only state; everything rendered comes fresh from the daemon each poll.
#[derive(Default)]
struct App {
    /// Which rate-limit policy ←/→ has selected.
    selected: usize,
    /// Whether the selected policy's detail pane is open.
    expanded: bool,
    /// Scroll offset within the detail pane.
    scroll: u16,
}

/// One `dashboard` response, unpacked.
struct Snap {
    pid: u32,
    version: String,
    provider: String,
    uptime_seconds: u64,
    connections: usize,
    logged_in: bool,
    username: Option<String>,
    access_expires_in_seconds: Option<u64>,
    keyring: String,
    in_flight: usize,
    max_in_flight: usize,
    policies: Vec<PolicyStatus>,
    policyless_endpoints: Vec<String>,
    degraded_endpoints: Vec<DegradedEndpoint>,
    jobs: Vec<JobInfo>,
    sends: Vec<SendRecord>,
    errors: Vec<ErrorRecord>,
}

pub async fn run(json: bool) -> Result<()> {
    let mut client = Client::connect(true).await?;
    if json {
        let resp = client.request(&Request::Dashboard).await?;
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }
    if !std::io::stdout().is_terminal() {
        bail!("stdout is not a terminal — use `acq dash --json` for a one-shot snapshot");
    }
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut client).await;
    ratatui::restore();
    result
}

async fn event_loop(terminal: &mut ratatui::DefaultTerminal, client: &mut Client) -> Result<()> {
    let mut app = App::default();
    loop {
        let snap = fetch(client).await?;
        terminal.draw(|f| draw(f, &snap, &mut app))?;
        if event::poll(POLL)? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    // Esc backs out of the detail pane first, then quits.
                    KeyCode::Esc if app.expanded => app.expanded = false,
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('e') => {
                        app.expanded = !app.expanded;
                        app.scroll = 0;
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                        let n = snap.policies.len().max(1);
                        app.selected = (app.selected + n - 1) % n;
                        app.scroll = 0;
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                        app.selected = (app.selected + 1) % snap.policies.len().max(1);
                        app.scroll = 0;
                    }
                    KeyCode::Up | KeyCode::Char('k') if app.expanded => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') if app.expanded => {
                        app.scroll = app.scroll.saturating_add(1);
                    }
                    KeyCode::PageUp if app.expanded => {
                        app.scroll = app.scroll.saturating_sub(DETAIL_ROWS);
                    }
                    KeyCode::PageDown if app.expanded => {
                        app.scroll = app.scroll.saturating_add(DETAIL_ROWS);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

async fn fetch(client: &mut Client) -> Result<Snap> {
    match client.request(&Request::Dashboard).await? {
        Response::Dashboard {
            pid,
            version,
            provider,
            uptime_seconds,
            connections,
            logged_in,
            username,
            access_expires_in_seconds,
            keyring,
            in_flight,
            max_in_flight,
            policies,
            policyless_endpoints,
            degraded_endpoints,
            jobs,
            sends,
            errors,
        } => Ok(Snap {
            pid,
            version,
            provider,
            uptime_seconds,
            connections,
            logged_in,
            username,
            access_expires_in_seconds,
            keyring,
            in_flight,
            max_in_flight,
            policies,
            policyless_endpoints,
            degraded_endpoints,
            jobs,
            sends,
            errors,
        }),
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

fn draw(f: &mut Frame, s: &Snap, app: &mut App) {
    app.selected = app.selected.min(s.policies.len().saturating_sub(1));
    // One summary line per policy, plus one for policyless endpoints (or a
    // placeholder when nothing has been learned yet).
    let summary_lines = s.policies.len().max(1) as u16
        + u16::from(!s.policyless_endpoints.is_empty())
        + s.degraded_endpoints.len() as u16;
    let policies_height = if app.expanded && !s.policies.is_empty() {
        2 + summary_lines + 1 + DETAIL_ROWS
    } else {
        2 + summary_lines
    };
    let [header, policies, jobs, bottom, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(policies_height),
        Constraint::Min(6),
        Constraint::Length(9),
        Constraint::Length(1),
    ])
    .areas(f.area());
    let [sends, errors] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(bottom);

    draw_header(f, header, s);
    draw_policies(f, policies, s, app);
    draw_jobs(f, jobs, &s.jobs);
    draw_sends(f, sends, &s.sends);
    draw_errors(f, errors, &s.errors);

    let hints = if app.expanded {
        "q quit · ←/→ policy · ↑/↓ scroll · enter/esc collapse · polling 4/s"
    } else {
        "q quit · ←/→ policy · enter expand policy detail · polling 4/s"
    };
    f.render_widget(Line::from(hints).dark_gray(), footer);
}

fn draw_header(f: &mut Frame, area: Rect, s: &Snap) {
    let provider = if s.provider == "ggg" {
        Span::styled("GGG (REAL)", Style::new().red().bold())
    } else {
        Span::styled(s.provider.clone(), Style::new().cyan())
    };
    let auth: Vec<Span> = if s.logged_in {
        let expiry = match s.access_expires_in_seconds {
            Some(sec) if sec > 0 => format!("token ~{}", fmt_secs(sec)),
            _ => "token expired (refreshes on use)".to_string(),
        };
        vec![
            Span::styled(
                format!("logged in as {}", s.username.as_deref().unwrap_or("<unknown>")),
                Style::new().green(),
            ),
            Span::raw(format!(" · {expiry} · keyring {}", s.keyring)),
        ]
    } else {
        vec![Span::styled("not logged in", Style::new().yellow())]
    };
    let mut spans = vec![
        Span::raw(format!("pid {} · v{} · ", s.pid, s.version)),
        provider,
        Span::raw(format!(
            " · up {} · {} conn{} · ",
            fmt_secs(s.uptime_seconds),
            s.connections,
            if s.connections == 1 { "" } else { "s" }
        )),
    ];
    spans.extend(auth);
    let block = Block::bordered().title(" acquisition daemon ");
    f.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

/// `hits/max·Ns` for one window, red when saturated, yellow when one away.
fn window_span(w: &acquisition_core::ratelimit::WindowStatus) -> Span<'static> {
    let style = if w.restricted_secs > 0 || w.hits >= w.max_hits {
        Style::new().red().bold()
    } else if w.hits + 1 >= w.max_hits {
        Style::new().yellow()
    } else {
        Style::new().green()
    };
    Span::styled(format!("{}/{}·{}s", w.hits, w.max_hits, w.period_secs), style)
}

fn next_span(p: &PolicyStatus) -> Span<'static> {
    if p.next_safe_in_seconds > 0.0 {
        Span::styled(format!("next in {:.1}s", p.next_safe_in_seconds), Style::new().red().bold())
    } else {
        Span::styled("ready", Style::new().green())
    }
}

fn draw_policies(f: &mut Frame, area: Rect, s: &Snap, app: &mut App) {
    let name_width = s.policies.iter().map(|p| p.policy.len()).max().unwrap_or(0).max(8);
    let mut summary: Vec<Line> = s
        .policies
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let selected = i == app.selected;
            let mut spans = vec![
                Span::styled(if selected { "▶ " } else { "  " }, Style::new().cyan()),
                Span::styled(
                    format!("{:<name_width$}  ", p.policy),
                    if selected { Style::new().bold() } else { Style::new() },
                ),
            ];
            for rule in &p.rules {
                spans.push(Span::styled(format!("{} ", rule.name.to_lowercase()), Style::new().dark_gray()));
                for (i, w) in rule.windows.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" "));
                    }
                    spans.push(window_span(w));
                }
                spans.push(Span::raw("  "));
            }
            spans.push(Span::raw("· "));
            spans.push(next_span(p));
            Line::from(spans)
        })
        .collect();
    if s.policies.is_empty() {
        summary.push(Line::from(Span::styled(
            "  no policies learned yet — the first response from each endpoint teaches the limiter",
            Style::new().dark_gray().italic(),
        )));
    }
    if !s.policyless_endpoints.is_empty() {
        summary.push(Line::from(vec![
            Span::styled("  no policy reported: ", Style::new().dark_gray()),
            Span::styled(s.policyless_endpoints.join(", "), Style::new().dark_gray().italic()),
        ]));
    }
    for d in &s.degraded_endpoints {
        summary.push(Line::from(vec![
            Span::styled(format!("  DEGRADED {} ({}s left): ", d.endpoint, d.seconds_left as u64), Style::new().red().bold()),
            Span::styled(d.reason.clone(), Style::new().red()),
        ]));
    }

    let in_flight_style = if s.in_flight >= s.max_in_flight { Style::new().yellow() } else { Style::new() };
    let block = Block::bordered().title(Line::from(vec![
        Span::raw(" rate limits (header-driven) · in flight "),
        Span::styled(format!("{}/{}", s.in_flight, s.max_in_flight), in_flight_style),
        Span::raw(" "),
    ]));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if !app.expanded || s.policies.is_empty() {
        f.render_widget(Paragraph::new(summary), inner);
        return;
    }

    let [summary_area, sep, detail_area] = Layout::vertical([
        Constraint::Length(summary.len() as u16),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);
    f.render_widget(Paragraph::new(summary), summary_area);

    let p = &s.policies[app.selected];
    let detail = policy_detail(p, &s.sends);
    let max_scroll = (detail.len() as u16).saturating_sub(detail_area.height);
    app.scroll = app.scroll.min(max_scroll);
    let mut sep_label = format!("─ {} ", p.policy);
    if max_scroll > 0 {
        sep_label.push_str(&format!("(↑/↓ scroll, {}/{}) ", app.scroll + 1, max_scroll + 1));
    }
    let pad = (sep.width as usize).saturating_sub(sep_label.chars().count());
    f.render_widget(
        Line::from(format!("{sep_label}{}", "─".repeat(pad))).dark_gray(),
        sep,
    );
    f.render_widget(Paragraph::new(detail).scroll((app.scroll, 0)), detail_area);
}

/// Everything known about one policy: what the server said, how the
/// limiter reads it, and the sends that taught it.
fn policy_detail(p: &PolicyStatus, sends: &[SendRecord]) -> Vec<Line<'static>> {
    let label = Style::new().dark_gray();
    let mut lines = vec![
        Line::from(vec![
            Span::styled("routes             ", label),
            Span::raw(p.endpoints.join(", ")),
        ]),
        Line::from(vec![
            Span::styled("next safe send     ", label),
            next_span(p),
            Span::raw(format!(
                " · last response {} ago · {} counted responses remembered",
                fmt_ago(p.last_observed_seconds_ago),
                p.history_len
            )),
        ]),
    ];
    if let Some(ra) = p.retry_after_secs {
        lines.push(Line::from(Span::styled(
            format!("retry-after        {ra}s on the last response — the server said WAIT (limiter adds the bucket, N19)"),
            Style::new().red().bold(),
        )));
    }
    lines.push(Line::from(""));
    for rule in &p.rules {
        lines.push(Line::from(vec![
            Span::styled("rule               ", label),
            Span::styled(rule.name.clone(), Style::new().bold()),
        ]));
        for (i, w) in rule.windows.iter().enumerate() {
            let kind = if i == 0 { "initial  " } else { "sustained" };
            let mut spans = vec![
                Span::styled(format!("  {kind}  "), label),
                window_span(w),
                Span::raw(format!(
                    "  (max {} per {}s, violation restricts {}s; padded by {}s bucket + 1s)",
                    w.max_hits, w.period_secs, w.restriction_secs, w.bucket_secs
                )),
            ];
            if w.restricted_secs > 0 {
                spans.push(Span::styled(
                    format!("  RESTRICTED for {}s", w.restricted_secs),
                    Style::new().red().bold(),
                ));
            }
            lines.push(Line::from(spans));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("raw headers (last response)", label)));
    let headers = p.headers.as_object().cloned().unwrap_or_default();
    for (name, value) in &headers {
        lines.push(Line::from(vec![
            Span::styled(format!("  {name}: "), label),
            Span::raw(value.as_str().unwrap_or_default().to_string()),
        ]));
    }

    lines.push(Line::from(""));
    let mine: Vec<&SendRecord> = sends.iter().filter(|r| p.endpoints.contains(&r.endpoint)).collect();
    if mine.is_empty() {
        lines.push(Line::from(Span::styled(
            "no requests sent under this policy yet",
            Style::new().dark_gray().italic(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("sends under this policy ({}, newest first)", mine.len()),
            label,
        )));
        for r in mine.iter().take(30) {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:>5}  ", fmt_ago(r.seconds_ago)), label),
                Span::raw(format!("{} ", r.method)),
                Span::styled(
                    r.outcome.clone(),
                    if r.ok { Style::new().green() } else { Style::new().red() },
                ),
                Span::styled(format!("  {}", r.url), label),
            ]));
        }
    }
    lines
}

fn draw_jobs(f: &mut Frame, area: Rect, jobs: &[JobInfo]) {
    let (mut waiting, mut running, mut done, mut failed, mut cancelled) = (0, 0, 0, 0, 0);
    for j in jobs {
        match j.state {
            JobState::Waiting => waiting += 1,
            JobState::Running => running += 1,
            JobState::Done => done += 1,
            JobState::Failed => failed += 1,
            JobState::Cancelled => cancelled += 1,
        }
    }
    let title = format!(
        " jobs — {running} running · {waiting} waiting · {done} done · {failed} failed · {cancelled} cancelled "
    );

    // Active work first (running, then the queue in dispatch order), then
    // finished jobs newest-first so fresh results stay visible.
    let mut sorted: Vec<&JobInfo> = jobs.iter().collect();
    sorted.sort_by_key(|j| match j.state {
        JobState::Running => (0u8, 0i64, j.id as i64),
        JobState::Waiting => (1, -(j.priority as i64), j.id as i64),
        _ => (2, 0, -(j.id as i64)),
    });

    let rows: Vec<Row> = sorted
        .iter()
        .map(|j| {
            let state_style = match j.state {
                JobState::Waiting => Style::new().yellow(),
                JobState::Running => Style::new().cyan().bold(),
                JobState::Done => Style::new().green(),
                JobState::Failed => Style::new().red().bold(),
                JobState::Cancelled => Style::new().dark_gray(),
            };
            let eta = match (j.state, j.eta_seconds) {
                (JobState::Waiting, Some(eta)) if eta > 0 => format!("~{}", fmt_secs(eta)),
                (JobState::Waiting, _) => "next".to_string(),
                _ => String::new(),
            };
            let kind_style = if j.kind == "probe" { Style::new().dark_gray() } else { Style::new() };
            Row::new(vec![
                Cell::from(j.id.to_string()),
                Cell::from(j.kind.clone()).style(kind_style),
                Cell::from(j.state.to_string()).style(state_style),
                Cell::from(j.priority.to_string()),
                Cell::from(j.submitted_by.clone()),
                Cell::from(eta),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(4),
            Constraint::Min(12),
            Constraint::Length(8),
        ],
    )
    .header(Row::new(vec!["id", "kind", "state", "prio", "by", "eta"]).dark_gray())
    .block(Block::bordered().title(title));
    f.render_widget(table, area);
}

fn draw_sends(f: &mut Frame, area: Rect, sends: &[SendRecord]) {
    let title = format!(" http sends ({}, newest first) ", sends.len());
    let rows: Vec<Row> = sends
        .iter()
        .map(|s| {
            let outcome_style = if s.ok {
                Style::new().green()
            } else {
                Style::new().red().bold()
            };
            Row::new(vec![
                Cell::from(fmt_ago(s.seconds_ago)).style(Style::new().dark_gray()),
                Cell::from(s.endpoint.clone()),
                Cell::from(s.method.clone()),
                Cell::from(s.outcome.clone()).style(outcome_style),
                Cell::from(s.url.clone()).style(Style::new().dark_gray()),
            ])
        })
        .collect();
    let empty = sends.is_empty();
    let mut table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(4),
            Constraint::Length(16),
            Constraint::Min(16),
        ],
    )
    .header(Row::new(vec!["age", "route", "", "outcome", "url"]).dark_gray())
    .block(Block::bordered().title(title));
    if empty {
        table = table.footer(Row::new(vec![Cell::from("nothing sent yet").dark_gray()]));
    }
    f.render_widget(table, area);
}

fn draw_errors(f: &mut Frame, area: Rect, errors: &[ErrorRecord]) {
    let title = format!(" errors ({}, newest first) ", errors.len());
    let lines: Vec<Line> = if errors.is_empty() {
        vec![Line::from(Span::styled("no errors", Style::new().green().dim()))]
    } else {
        errors
            .iter()
            .map(|e| {
                Line::from(vec![
                    Span::styled(format!("{:>6}  ", fmt_ago(e.seconds_ago)), Style::new().dark_gray()),
                    Span::styled(e.message.clone(), Style::new().red()),
                ])
            })
            .collect()
    };
    let block = Block::bordered().title(Span::styled(
        title,
        if errors.is_empty() { Style::new() } else { Style::new().red() },
    ));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(block), area);
}

fn fmt_secs(s: u64) -> String {
    if s >= 3600 {
        format!("{}h{:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

fn fmt_ago(s: f64) -> String {
    if s < 1.0 { "<1s".to_string() } else { fmt_secs(s as u64) }
}
