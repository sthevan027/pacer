pub mod api;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use api::{ApiError, ApiWindow};
use crate::secrets::SecretStore;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

pub const PROVIDER_ID: &str = "openrouter";
const KEY_ENV: &str = "OPENROUTER_API_KEY";

/// Função que devolve a chave, ou `None` se não há. Injetável para os testes não mexerem no
/// ambiente do processo.
pub type KeySource = Box<dyn Fn() -> Option<String> + Send + Sync>;

/// Variável de ambiente do usuário (o padrão das ferramentas do OpenRouter).
pub fn env_key() -> Option<String> {
    std::env::var(KEY_ENV).ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty())
}

/// A chave colada no Pacer (Gerenciador de Credenciais) vale mais que a variável de ambiente.
pub fn key_from(store: Arc<dyn SecretStore>, env: fn() -> Option<String>) -> KeySource {
    Box::new(move || store.get(PROVIDER_ID).or_else(env))
}

/// Período já virado desde a última busca: o dado antigo não vale mais.
fn to_windows(api: &[ApiWindow], now: DateTime<Utc>) -> Vec<UsageWindow> {
    api.iter()
        .map(|w| {
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

pub struct OpenRouterProvider {
    key: KeySource,
    url: String,
    http: reqwest::Client,
    plan: Option<String>,
    api_windows: Vec<ApiWindow>,
    windows: Vec<UsageWindow>,
    fetched_at: Option<DateTime<Utc>>,
    stale: bool,
    notice: Option<Notice>,
}

impl OpenRouterProvider {
    pub fn new(store: Arc<dyn SecretStore>) -> Self {
        Self::with(key_from(store, env_key), api::KEY_URL.into())
    }

    pub fn with(key: KeySource, url: String) -> Self {
        Self {
            key,
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
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        self.windows = to_windows(&self.api_windows, now);
    }

    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        let Some(key) = (self.key)() else {
            self.api_windows.clear();
            self.windows.clear();
            self.plan = None;
            self.stale = false;
            self.notice = Some(Notice::NoCredentials);
            return FetchResult::Skipped;
        };
        match api::fetch_key(&self.http, &self.url, &key, now).await {
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
            name: "OpenRouter".into(),
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

    const BODY: &str = r#"{"data":{"limit":50.0,"limit_remaining":38.0,"limit_reset":"monthly","usage_monthly":12.0}}"#;

    async fn server(status: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/key").with_status(status).with_body(BODY).create_async().await;
        let url = format!("{}/key", s.url());
        (s, url)
    }

    fn with_key(url: String) -> OpenRouterProvider {
        OpenRouterProvider::with(Box::new(|| Some("k".into())), url)
    }

    #[tokio::test]
    async fn without_a_key_shows_notice_and_never_calls_the_api() {
        let mut server = mockito::Server::new_async().await;
        let never = server.mock("GET", "/key").expect(0).create_async().await;
        let mut p = OpenRouterProvider::with(Box::new(|| None), format!("{}/key", server.url()));
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        never.assert_async().await;
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
    }

    #[tokio::test]
    async fn successful_fetch_fills_bar_and_spend_badge() {
        let (_s, url) = server(200).await;
        let mut p = with_key(url);
        let now = Utc::now();
        assert_eq!(p.fetch_remote(now).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!((s.provider.as_str(), s.name.as_str()), ("openrouter", "OpenRouter"));
        assert_eq!(s.plan.as_deref(), Some("US$ 12.00 no mês"));
        assert_eq!(s.windows.len(), 1);
        assert_eq!(s.windows[0].used_pct, 24.0);
        assert_eq!((s.fetched_at, s.notice, s.stale), (Some(now), None, false));
    }

    #[tokio::test]
    async fn failures_after_a_success_keep_the_data_as_stale() {
        let (_ok, url) = server(200).await;
        let mut p = with_key(url);
        p.fetch_remote(Utc::now()).await;
        for (code, notice, result) in [
            (429, Notice::RateLimited, FetchResult::RateLimited),
            (401, Notice::TokenExpired, FetchResult::Failed),
            (500, Notice::Offline, FetchResult::Failed),
        ] {
            let (_bad, bad_url) = server(code).await;
            p.url = bad_url;
            assert_eq!(p.fetch_remote(Utc::now()).await, result, "{code}");
            let s = p.snapshot();
            assert_eq!((s.notice, s.stale, s.windows.len()), (Some(notice), true, 1), "{code}");
        }
    }

    #[tokio::test]
    async fn a_period_that_rolled_over_counts_as_zero() {
        let (_s, url) = server(200).await;
        let mut p = with_key(url);
        let now = Utc::now();
        p.fetch_remote(now).await;
        p.tick_local(now + chrono::Duration::days(45));
        let s = p.snapshot();
        assert_eq!((s.windows[0].used_pct, s.windows[0].resets_at), (0.0, None));
    }

    #[test]
    fn a_pasted_key_wins_over_the_environment() {
        let store = Arc::new(crate::secrets::MemoryStore::default());
        let from = |s: &Arc<crate::secrets::MemoryStore>, env: fn() -> Option<String>| key_from(s.clone(), env)();
        assert_eq!(from(&store, || Some("do-ambiente".into())).as_deref(), Some("do-ambiente"));
        assert_eq!(from(&store, || None), None);
        store.set(PROVIDER_ID, "colada").unwrap();
        assert_eq!(from(&store, || Some("do-ambiente".into())).as_deref(), Some("colada"));
    }
}
