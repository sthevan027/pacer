//! Cota do GitHub Copilot. O endpoint `copilot_internal/user` não é documentado (é o que o
//! próprio Copilot e ferramentas como o CodexBar usam), então o parser aceita só o que entende
//! e ignora o resto. A rota oficial de cobrança exige escopo `user` e não traz a cota restante.

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

pub const USER_URL: &str = "https://api.github.com/copilot_internal/user";

#[derive(Deserialize)]
struct RawQuota {
    percent_remaining: Option<f64>,
    unlimited: Option<bool>,
    has_quota: Option<bool>,
}

#[derive(Deserialize)]
struct RawSnapshots {
    chat: Option<RawQuota>,
    completions: Option<RawQuota>,
    premium_interactions: Option<RawQuota>,
}

#[derive(Deserialize)]
struct RawUser {
    copilot_plan: Option<String>,
    access_type_sku: Option<String>,
    quota_snapshots: Option<RawSnapshots>,
    quota_reset_date_utc: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiWindow {
    pub id: &'static str,
    pub label: &'static str,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub length: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    pub plan: Option<String>,
    pub windows: Vec<ApiWindow>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Unauthorized,
    /// 404: a conta não tem Copilot (ou o endpoint mudou).
    NoCopilot,
    RateLimited,
    Network(String),
    BadResponse(String),
}

/// `free_limited_copilot` → "Free"; o resto vem do `copilot_plan` ("business" → "Business").
fn plan_name(sku: Option<&str>, plan: Option<&str>) -> Option<String> {
    if sku.is_some_and(|s| s.to_ascii_lowercase().contains("free")) {
        return Some("Free".into());
    }
    let mut c = plan?.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str())
}

/// Cota sem limite, sem direito (`has_quota: false`, como o premium do plano Free) ou sem
/// percentual não vira barra.
fn window(id: &'static str, label: &'static str, q: Option<RawQuota>, resets_at: Option<DateTime<Utc>>) -> Option<ApiWindow> {
    let q = q?;
    if q.unlimited == Some(true) || q.has_quota == Some(false) {
        return None;
    }
    let remaining = q.percent_remaining?;
    if !remaining.is_finite() {
        return None;
    }
    // A cota é mensal; o mês exato não vem na resposta, 30 dias basta para a previsão de ritmo.
    Some(ApiWindow { id, label, used_pct: (100.0 - remaining).clamp(0.0, 100.0), resets_at, length: Duration::days(30) })
}

pub fn parse_user(body: &str) -> Result<Usage, serde_json::Error> {
    let raw: RawUser = serde_json::from_str(body)?;
    let reset = raw.quota_reset_date_utc;
    let (chat, completions, premium) = match raw.quota_snapshots {
        Some(s) => (s.chat, s.completions, s.premium_interactions),
        None => (None, None, None),
    };
    let windows = [
        window("chat", "Chat · mensal", chat, reset),
        window("completions", "Sugestões · mensal", completions, reset),
        window("premium", "Premium · mensal", premium, reset),
    ]
    .into_iter()
    .flatten()
    .collect();
    Ok(Usage { plan: plan_name(raw.access_type_sku.as_deref(), raw.copilot_plan.as_deref()), windows })
}

pub async fn fetch_user(client: &reqwest::Client, url: &str, token: &str) -> Result<Usage, ApiError> {
    let resp = client
        .get(url)
        .header("Authorization", format!("token {token}"))
        .header("Accept", "application/json")
        .header("User-Agent", "pacer")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        401 | 403 => return Err(ApiError::Unauthorized),
        404 => return Err(ApiError::NoCopilot),
        429 => return Err(ApiError::RateLimited),
        s if !(200..300).contains(&s) => return Err(ApiError::BadResponse(format!("HTTP {s}"))),
        _ => {}
    }
    let body = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;
    parse_user(&body).map_err(|e| ApiError::BadResponse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const FREE: &str = include_str!("../../../tests/fixtures/copilot_user.json");

    #[test]
    fn free_plan_gives_chat_and_completions_without_the_empty_premium_quota() {
        let u = parse_user(FREE).unwrap();
        assert_eq!(u.plan.as_deref(), Some("Free"));
        let ids: Vec<_> = u.windows.iter().map(|w| w.id).collect();
        assert_eq!(ids, ["chat", "completions"]);
        assert!((u.windows[0].used_pct - 26.4).abs() < 1e-9);
        assert!((u.windows[1].used_pct - 0.4).abs() < 1e-9);
        assert_eq!(u.windows[0].resets_at, Some(Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap()));
        assert_eq!(u.windows[0].label, "Chat · mensal");
    }

    #[test]
    fn paid_plan_shows_premium_and_skips_unlimited_quotas() {
        let body = r#"{"copilot_plan":"business","access_type_sku":"copilot_for_business_seat",
            "quota_snapshots":{
              "chat":{"percent_remaining":100.0,"unlimited":true,"has_quota":true},
              "completions":{"percent_remaining":100.0,"unlimited":true,"has_quota":true},
              "premium_interactions":{"percent_remaining":40.0,"unlimited":false,"has_quota":true}}}"#;
        let u = parse_user(body).unwrap();
        assert_eq!(u.plan.as_deref(), Some("Business"));
        assert_eq!(u.windows.len(), 1);
        assert_eq!((u.windows[0].id, u.windows[0].used_pct), ("premium", 60.0));
        assert_eq!(u.windows[0].resets_at, None);
    }

    #[test]
    fn missing_or_odd_quota_data_gives_no_windows_instead_of_failing() {
        assert!(parse_user(r#"{"copilot_plan":"individual"}"#).unwrap().windows.is_empty());
        assert!(parse_user(r#"{"quota_snapshots":{"chat":{"unlimited":false}}}"#).unwrap().windows.is_empty());
        assert!(parse_user("{}").unwrap().plan.is_none());
    }

    #[test]
    fn percentages_are_clamped() {
        let u = parse_user(r#"{"quota_snapshots":{"chat":{"percent_remaining":-20.0},"completions":{"percent_remaining":150.0}}}"#).unwrap();
        assert_eq!((u.windows[0].used_pct, u.windows[1].used_pct), (100.0, 0.0));
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse_user("não é json").is_err());
    }

    async fn server(code: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/user").match_header("authorization", "token tok").with_status(code).with_body(FREE).create_async().await;
        let url = format!("{}/user", s.url());
        (s, url)
    }

    #[tokio::test]
    async fn fetch_sends_the_token_and_parses() {
        let (_s, url) = server(200).await;
        let u = fetch_user(&reqwest::Client::new(), &url, "tok").await.unwrap();
        assert_eq!(u.windows.len(), 2);
    }

    #[tokio::test]
    async fn http_statuses_map_to_errors() {
        for (code, want) in [(401, ApiError::Unauthorized), (403, ApiError::Unauthorized), (404, ApiError::NoCopilot), (429, ApiError::RateLimited)] {
            let (_s, url) = server(code).await;
            assert_eq!(fetch_user(&reqwest::Client::new(), &url, "tok").await.unwrap_err(), want, "{code}");
        }
        let (_s, url) = server(500).await;
        assert!(matches!(fetch_user(&reqwest::Client::new(), &url, "tok").await, Err(ApiError::BadResponse(_))));
    }

    #[tokio::test]
    async fn fetch_network_error() {
        let r = fetch_user(&reqwest::Client::new(), "http://127.0.0.1:9/user", "tok").await;
        assert!(matches!(r, Err(ApiError::Network(_))));
    }
}
