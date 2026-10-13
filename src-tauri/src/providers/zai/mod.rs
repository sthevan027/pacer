pub mod api;
pub mod creds;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use api::{ApiError, ApiWindow};
use crate::secrets::SecretStore;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use creds::Auth;
use std::sync::Arc;

pub const PROVIDER_ID: &str = "zai";

/// Função que devolve o login atual, ou `None`. Injetável para os testes não lerem credencial real.
pub type AuthSource = Box<dyn Fn() -> Option<Auth> + Send + Sync>;

/// Ciclo já virado desde a última busca: o dado antigo não vale mais.
fn to_windows(api: &[ApiWindow], now: DateTime<Utc>) -> Vec<UsageWindow> {
    api.iter()
        .map(|w| {
            let expired = w.resets_at.is_some_and(|r| r <= now);
            let used_pct = if expired { 0.0 } else { w.used_pct };
            let resets_at = if expired { None } else { w.resets_at };
            UsageWindow {
                id: w.id.clone(),
                label: w.label.clone(),
                used_pct,
                resets_at,
                pace: resets_at.and_then(|r| pacing::project(used_pct, w.length, r, now)),
            }
        })
        .collect()
}

pub struct ZaiProvider {
    auth: AuthSource,
    /// Só os testes trocam: em produção a URL sai do domínio do login.
    url_override: Option<String>,
    http: reqwest::Client,
    api_windows: Vec<ApiWindow>,
    windows: Vec<UsageWindow>,
    fetched_at: Option<DateTime<Utc>>,
    stale: bool,
    notice: Option<Notice>,
}

impl ZaiProvider {
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self::with(Box::new(move || creds::stored_or_discovered(&*store)), None)
    }

    pub fn with(auth: AuthSource, url_override: Option<String>) -> Self {
        Self {
            auth,
            url_override,
            http: reqwest::Client::new(),
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
impl Provider for ZaiProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        self.windows = to_windows(&self.api_windows, now);
    }

    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        let Some(auth) = (self.auth)() else {
            self.api_windows.clear();
            self.windows.clear();
            self.stale = false;
            self.notice = Some(Notice::NoCredentials);
            return FetchResult::Skipped;
        };
        let url = self.url_override.clone().unwrap_or_else(|| format!("{}{}", auth.domain, api::QUOTA_PATH));
        match api::fetch_quota(&self.http, &url, &auth.token).await {
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
            provider: PROVIDER_ID.into(),
            name: "z.ai GLM".into(),
            plan: None,
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

    const BODY: &str = r#"{"data":{"limits":[{"type":"TOKENS_LIMIT","percentage":37.0},{"type":"TIME_LIMIT","percentage":12.0}]}}"#;

    async fn server(status: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/q").with_status(status).with_body(BODY).create_async().await;
        let url = format!("{}/q", s.url());
        (s, url)
    }

    fn with_auth(url: String) -> ZaiProvider {
        ZaiProvider::with(Box::new(|| Some(Auth { token: "tok".into(), domain: creds::ZAI.into() })), Some(url))
    }

    #[tokio::test]
    async fn without_a_login_shows_notice_and_never_calls_the_api() {
        let mut server = mockito::Server::new_async().await;
        let never = server.mock("GET", "/q").expect(0).create_async().await;
        let mut p = ZaiProvider::with(Box::new(|| None), Some(format!("{}/q", server.url())));
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        never.assert_async().await;
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
    }

    #[tokio::test]
    async fn successful_fetch_fills_the_two_bars() {
        let (_s, url) = server(200).await;
        let mut p = with_auth(url);
        let now = Utc::now();
        assert_eq!(p.fetch_remote(now).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!((s.provider.as_str(), s.name.as_str()), ("zai", "z.ai GLM"));
        assert_eq!(s.windows.len(), 2);
        assert_eq!((s.windows[0].id.as_str(), s.windows[0].used_pct), ("five_hour", 37.0));
        assert_eq!((s.fetched_at, s.notice, s.stale), (Some(now), None, false));
    }

    #[tokio::test]
    async fn failures_after_a_success_keep_the_data_as_stale() {
        let (_ok, url) = server(200).await;
        let mut p = with_auth(url);
        p.fetch_remote(Utc::now()).await;
        for (code, notice, result) in [
            (429, Notice::RateLimited, FetchResult::RateLimited),
            (401, Notice::TokenExpired, FetchResult::Failed),
            (500, Notice::Offline, FetchResult::Failed),
        ] {
            let (_bad, bad_url) = server(code).await;
            p.url_override = Some(bad_url);
            assert_eq!(p.fetch_remote(Utc::now()).await, result, "{code}");
            let s = p.snapshot();
            assert_eq!((s.notice, s.stale, s.windows.len()), (Some(notice), true, 2), "{code}");
        }
    }

    #[test]
    fn production_url_comes_from_the_login_domain() {
        let a = Auth { token: "t".into(), domain: creds::ZHIPU.into() };
        assert_eq!(format!("{}{}", a.domain, api::QUOTA_PATH), "https://open.bigmodel.cn/api/monitor/usage/quota/limit");
    }
}
