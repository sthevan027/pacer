pub mod api;
pub mod creds;
pub mod logs;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use crate::usage::{Activity, UsageStore};
use api::{ApiError, ApiWindow};
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use creds::CredsError;
use logs::LogScanner;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Versão do Claude Code usada no User-Agent (o endpoint exige esse formato).
pub const CLAUDE_CLI_VERSION: &str = "2.1.288";

fn to_windows(api: &[ApiWindow], now: DateTime<Utc>) -> Vec<UsageWindow> {
    api.iter()
        .map(|w| UsageWindow {
            id: w.id.into(),
            label: w.label.into(),
            used_pct: w.used_pct,
            resets_at: w.resets_at,
            pace: w.resets_at.and_then(|r| pacing::project(w.used_pct, w.length, r, now)),
        })
        .collect()
}

pub struct ClaudeProvider {
    claude_dir: PathBuf,
    usage_url: String,
    http: reqwest::Client,
    scanner: LogScanner,
    /// O mesmo store que o comando do painel lê. Antes o scanner morava só aqui dentro, e o
    /// painel não teria como alcançá-lo.
    usage: Arc<Mutex<UsageStore>>,
    activity: Activity,
    plan: Option<String>,
    api_windows: Vec<ApiWindow>,
    windows: Vec<UsageWindow>,
    fetched_at: Option<DateTime<Utc>>,
    stale: bool,
    notice: Option<Notice>,
}

impl ClaudeProvider {
    pub fn new(claude_dir: PathBuf, usage: Arc<Mutex<UsageStore>>) -> Self {
        Self::with_url(claude_dir, api::USAGE_URL.into(), usage)
    }

    pub fn with_url(claude_dir: PathBuf, usage_url: String, usage: Arc<Mutex<UsageStore>>) -> Self {
        Self {
            claude_dir,
            usage_url,
            http: reqwest::Client::new(),
            scanner: LogScanner::new(),
            usage,
            activity: Activity { days: Vec::new(), today_tokens: 0 },
            plan: None,
            api_windows: Vec::new(),
            windows: Vec::new(),
            fetched_at: None,
            stale: false,
            notice: None,
        }
    }

    fn mark_failure(&mut self, notice: Notice) {
        self.stale = !self.api_windows.is_empty();
        self.notice = Some(notice);
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        let events = self.scanner.scan(&self.claude_dir.join("projects"), now.with_timezone(&Local));
        let mut store = self.usage.lock().unwrap();
        store.set_provider(logs::PROVIDER_ID, events);
        self.activity = store.activity(now.with_timezone(&Local));
        drop(store);
        self.windows = to_windows(&self.api_windows, now);
    }

    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        let creds = match creds::read(&self.claude_dir) {
            Ok(c) => c,
            Err(CredsError::Missing) => {
                self.api_windows.clear();
                self.windows.clear();
                self.plan = None;
                self.stale = false;
                self.notice = Some(Notice::NoCredentials);
                return FetchResult::Skipped;
            }
            // Claude Code reescrevendo o arquivo agora: mantém tudo como está.
            Err(CredsError::Unreadable(_)) => return FetchResult::Failed,
        };
        self.plan = creds.plan.clone();
        if creds.is_expired(now) {
            self.mark_failure(Notice::TokenExpired);
            return FetchResult::Skipped;
        }
        let ua = api::user_agent(CLAUDE_CLI_VERSION);
        match api::fetch_usage(&self.http, &self.usage_url, &creds.access_token, &ua).await {
            Ok(ws) => {
                self.api_windows = ws;
                self.windows = to_windows(&self.api_windows, now);
                self.fetched_at = Some(now);
                self.stale = false;
                self.notice = None;
                FetchResult::Ok
            }
            Err(ApiError::Unauthorized) => {
                self.mark_failure(Notice::TokenExpired);
                FetchResult::Failed
            }
            Err(ApiError::RateLimited) => {
                self.mark_failure(Notice::RateLimited);
                FetchResult::RateLimited
            }
            Err(ApiError::Network(_)) | Err(ApiError::BadResponse(_)) => {
                self.mark_failure(Notice::Offline);
                FetchResult::Failed
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            provider: "claude".into(),
            name: "Claude".into(),
            plan: self.plan.clone(),
            windows: self.windows.clone(),
            activity: self.activity.days.clone(),
            today_tokens: self.activity.today_tokens,
            fetched_at: self.fetched_at,
            next_refresh_at: None,
            stale: self.stale,
            notice: self.notice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{FetchResult, Provider};
    use crate::snapshot::Notice;
    use chrono::Utc;
    use std::fs;

    const MINIMAL: &str = include_str!("../../../tests/fixtures/usage_minimal.json");

    fn test_provider(dir: &std::path::Path, usage_url: String) -> ClaudeProvider {
        ClaudeProvider::with_url(dir.into(), usage_url, Arc::new(Mutex::new(UsageStore::new())))
    }

    fn write_creds(dir: &std::path::Path, expires_in_hours: i64) {
        let exp = (Utc::now() + chrono::Duration::hours(expires_in_hours)).timestamp_millis();
        fs::write(
            dir.join(".credentials.json"),
            format!(r#"{{"claudeAiOauth":{{"accessToken":"tok","expiresAt":{exp},"subscriptionType":"pro"}}}}"#),
        )
        .unwrap();
    }

    async fn server_with(status: usize) -> mockito::ServerGuard {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/usage").with_status(status).with_body(MINIMAL).create_async().await;
        s
    }

    #[tokio::test]
    async fn without_credentials_shows_notice_and_skips() {
        let d = tempfile::tempdir().unwrap();
        let mut p = test_provider(d.path(), "http://127.0.0.1:9/usage".into());
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
        assert_eq!(s.fetched_at, None);
    }

    #[tokio::test]
    async fn successful_fetch_fills_windows_plan_and_pace() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let server = server_with(200).await;
        let mut p = test_provider(d.path(), format!("{}/usage", server.url()));
        let now = Utc::now();
        assert_eq!(p.fetch_remote(now).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!(s.plan.as_deref(), Some("Pro"));
        assert_eq!(s.windows.len(), 2);
        assert_eq!(s.windows[0].label, "Sessão (5h)");
        assert_eq!(s.fetched_at, Some(now));
        assert!(!s.stale);
        assert_eq!(s.notice, None);
    }

    #[tokio::test]
    async fn rate_limit_after_success_keeps_data_as_stale() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let ok = server_with(200).await;
        let mut p = test_provider(d.path(), format!("{}/usage", ok.url()));
        p.fetch_remote(Utc::now()).await;
        let limited = server_with(429).await;
        p.usage_url = format!("{}/usage", limited.url());
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::RateLimited);
        let s = p.snapshot();
        assert_eq!(s.windows.len(), 2);
        assert!(s.stale);
        assert_eq!(s.notice, Some(Notice::RateLimited));
    }

    #[tokio::test]
    async fn half_written_credentials_keep_last_state_without_login_notice() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let ok = server_with(200).await;
        let mut p = test_provider(d.path(), format!("{}/usage", ok.url()));
        p.fetch_remote(Utc::now()).await;
        fs::write(d.path().join(".credentials.json"), r#"{"claudeAiOauth":{"acc"#).unwrap();
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Failed);
        let s = p.snapshot();
        assert_eq!(s.windows.len(), 2);
        assert_eq!(s.notice, None);
        assert_eq!(s.plan.as_deref(), Some("Pro"));
    }

    #[tokio::test]
    async fn expired_token_shows_notice_without_calling_api() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), -1);
        let mut server = mockito::Server::new_async().await;
        let never = server.mock("GET", "/usage").expect(0).create_async().await;
        let mut p = test_provider(d.path(), format!("{}/usage", server.url()));
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        never.assert_async().await;
        assert_eq!(p.snapshot().notice, Some(Notice::TokenExpired));
    }

    #[tokio::test]
    async fn tick_local_reads_logs_and_feeds_the_shared_store() {
        let d = tempfile::tempdir().unwrap();
        let proj = d.path().join("projects").join("x");
        fs::create_dir_all(&proj).unwrap();
        let ts = Utc::now().to_rfc3339();
        fs::write(
            proj.join("s.jsonl"),
            format!(
                r#"{{"type":"assistant","timestamp":"{ts}","sessionId":"sess","cwd":"D:\\Projetos\\pacer","requestId":"r","message":{{"id":"m","model":"claude-opus-5-5","usage":{{"input_tokens":42}}}}}}"#
            ),
        )
        .unwrap();

        // o mesmo store que o painel vai ler pelo comando
        let usage = Arc::new(Mutex::new(UsageStore::new()));
        let mut p = ClaudeProvider::with_url(d.path().into(), "http://127.0.0.1:9/usage".into(), usage.clone());
        p.tick_local(Utc::now());

        let s = p.snapshot();
        assert_eq!(s.activity.len(), 30);
        assert_eq!(s.today_tokens, 42);

        // e o painel enxerga o mesmo evento, com os campos ricos
        let store = usage.lock().unwrap();
        let report = store.report(&crate::usage::ReportFilter::default(), Local::now(), 5.0);
        assert_eq!(report.totals.input, 42);
        assert_eq!(report.by_model[0].model, "claude-opus-5-5");
        assert_eq!(report.top_projects[0].name, "pacer");
    }
}
