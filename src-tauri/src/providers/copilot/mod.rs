pub mod api;
pub mod creds;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use api::{ApiError, ApiWindow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use creds::TokenSource;

pub const PROVIDER_ID: &str = "copilot";

fn to_windows(api: &[ApiWindow], now: DateTime<Utc>) -> Vec<UsageWindow> {
    api.iter()
        .map(|w| {
            // cota já renovada desde a última busca: o dado antigo não vale mais
            let expired = w.resets_at.is_some_and(|r| r <= now);
            let used_pct = if expired { 0.0 } else { w.used_pct };
            let resets_at = if expired { None } else { w.resets_at };
            UsageWindow {
                id: w.id.into(),
                label: w.label.into(),
                used_pct,
                resets_at,
                pace: resets_at.and_then(|r| pacing::project(used_pct, w.length, r, now)),
            }
        })
        .collect()
}

pub struct CopilotProvider {
    token: TokenSource,
    url: String,
    http: reqwest::Client,
    plan: Option<String>,
    api_windows: Vec<ApiWindow>,
    windows: Vec<UsageWindow>,
    fetched_at: Option<DateTime<Utc>>,
    stale: bool,
    notice: Option<Notice>,
}

impl CopilotProvider {
    pub fn new() -> Self {
        Self::with(Box::new(creds::gh_token), api::USER_URL.into())
    }

    pub fn with(token: TokenSource, url: String) -> Self {
        Self {
            token,
            url,
            http: reqwest::Client::new(),
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

    fn clear(&mut self, notice: Notice) {
        self.api_windows.clear();
        self.windows.clear();
        self.plan = None;
        self.stale = false;
        self.notice = Some(notice);
    }
}

#[async_trait]
impl Provider for CopilotProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        self.windows = to_windows(&self.api_windows, now);
    }

    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        // `gh auth token` é um processo curto (dezenas de ms) a cada refresh de 1–10 min.
        let Some(token) = (self.token)() else {
            self.clear(Notice::NoCredentials);
            return FetchResult::Skipped;
        };
        match api::fetch_user(&self.http, &self.url, &token).await {
            Ok(u) => {
                self.plan = u.plan;
                self.api_windows = u.windows;
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
            Err(ApiError::NoCopilot) => {
                self.clear(Notice::NoCredentials);
                FetchResult::Skipped
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
            provider: PROVIDER_ID.into(),
            name: "GitHub Copilot".into(),
            plan: self.plan.clone(),
            windows: self.windows.clone(),
            activity: Vec::new(),
            today_tokens: 0,
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
    use chrono::TimeZone;

    const FREE: &str = include_str!("../../../tests/fixtures/copilot_user.json");

    fn before_reset() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap()
    }

    fn with_token(url: String) -> CopilotProvider {
        CopilotProvider::with(Box::new(|| Some("tok".into())), url)
    }

    async fn server(status: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/user").with_status(status).with_body(FREE).create_async().await;
        let url = format!("{}/user", s.url());
        (s, url)
    }

    #[tokio::test]
    async fn without_a_gh_login_shows_notice_and_never_calls_the_api() {
        let mut server = mockito::Server::new_async().await;
        let never = server.mock("GET", "/user").expect(0).create_async().await;
        let mut p = CopilotProvider::with(Box::new(|| None), format!("{}/user", server.url()));
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        never.assert_async().await;
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
    }

    #[tokio::test]
    async fn successful_fetch_fills_plan_and_monthly_windows() {
        let (_s, url) = server(200).await;
        let mut p = with_token(url);
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!(s.provider, "copilot");
        assert_eq!(s.name, "GitHub Copilot");
        assert_eq!(s.plan.as_deref(), Some("Free"));
        assert_eq!(s.windows.len(), 2);
        assert!((s.windows[0].used_pct - 26.4).abs() < 1e-9);
        assert_eq!(s.fetched_at, Some(before_reset()));
        assert_eq!(s.notice, None);
        assert!(!s.stale);
    }

    #[tokio::test]
    async fn failures_after_a_success_keep_the_data_as_stale() {
        let (_ok, url) = server(200).await;
        let mut p = with_token(url);
        p.fetch_remote(before_reset()).await;
        for (code, notice, result) in [
            (429, Notice::RateLimited, FetchResult::RateLimited),
            (401, Notice::TokenExpired, FetchResult::Failed),
            (500, Notice::Offline, FetchResult::Failed),
        ] {
            let (_bad, bad_url) = server(code).await;
            p.url = bad_url;
            assert_eq!(p.fetch_remote(before_reset()).await, result, "{code}");
            let s = p.snapshot();
            assert_eq!((s.notice, s.stale, s.windows.len()), (Some(notice), true, 2), "{code}");
        }
    }

    #[tokio::test]
    async fn account_without_copilot_clears_the_card() {
        let (_ok, url) = server(200).await;
        let mut p = with_token(url);
        p.fetch_remote(before_reset()).await;
        let (_nf, nf_url) = server(404).await;
        p.url = nf_url;
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert!(s.windows.is_empty());
        assert_eq!((s.notice, s.plan), (Some(Notice::NoCredentials), None));
    }

    #[tokio::test]
    async fn quota_renewed_since_the_last_fetch_counts_as_zero() {
        let (_s, url) = server(200).await;
        let mut p = with_token(url);
        p.fetch_remote(before_reset()).await;
        p.tick_local(Utc.with_ymd_and_hms(2026, 11, 2, 0, 0, 0).unwrap());
        let s = p.snapshot();
        assert_eq!(s.windows[0].used_pct, 0.0);
        assert_eq!(s.windows[0].resets_at, None);
    }
}
