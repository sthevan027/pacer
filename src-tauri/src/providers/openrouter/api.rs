//! Uso da chave do OpenRouter (`GET /api/v1/key`, documentado, aceita chave comum; o `/credits`
//! exige chave de gerenciamento e por isso não é usado). Campos conferidos em
//! <https://openrouter.ai/docs/api-reference/limits>.

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde::Deserialize;

pub const KEY_URL: &str = "https://openrouter.ai/api/v1/key";

#[derive(Deserialize)]
struct RawFree {
    used: Option<f64>,
    limit: Option<f64>,
}

#[derive(Deserialize)]
struct RawKey {
    limit: Option<f64>,
    limit_remaining: Option<f64>,
    limit_reset: Option<String>,
    usage_monthly: Option<f64>,
    free_model_daily_requests: Option<RawFree>,
}

#[derive(Deserialize)]
struct RawResponse {
    data: Option<RawKey>,
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
    /// "US$ 12.34 no mês": o gasto em créditos vira o selo do card (chave sem limite não tem barra).
    pub plan: Option<String>,
    pub windows: Vec<ApiWindow>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Unauthorized,
    RateLimited,
    Network(String),
    BadResponse(String),
}

/// Início e próxima redefinição (UTC) do período do OpenRouter: dia UTC, semana a partir da
/// segunda, mês UTC. `None` para um tipo de reset que não conhecemos.
fn period(kind: &str, now: DateTime<Utc>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let midnight = |d: chrono::NaiveDate| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap());
    let today = now.date_naive();
    match kind {
        "daily" => Some((midnight(today), midnight(today) + Duration::days(1))),
        "weekly" => {
            let start = midnight(today - Duration::days(i64::from(today.weekday().num_days_from_monday())));
            Some((start, start + Duration::days(7)))
        }
        "monthly" => {
            let start = midnight(today.with_day(1)?);
            let next = if today.month() == 12 { today.with_year(today.year() + 1)?.with_month(1)? } else { today.with_month(today.month() + 1)? };
            Some((start, midnight(next.with_day(1)?)))
        }
        _ => None,
    }
}

pub fn parse_key(body: &str, now: DateTime<Utc>) -> Result<Usage, serde_json::Error> {
    let raw: RawResponse = serde_json::from_str(body)?;
    let Some(k) = raw.data else {
        return Ok(Usage { plan: None, windows: Vec::new() });
    };
    let mut windows = Vec::new();

    if let (Some(limit), Some(remaining)) = (k.limit, k.limit_remaining) {
        if limit > 0.0 && limit.is_finite() && remaining.is_finite() {
            let span = k.limit_reset.as_deref().and_then(|r| period(r, now));
            windows.push(ApiWindow {
                id: "key_limit",
                label: "Limite da chave",
                used_pct: ((limit - remaining) / limit * 100.0).clamp(0.0, 100.0),
                resets_at: span.map(|s| s.1),
                length: span.map_or(Duration::days(30), |s| s.1 - s.0),
            });
        }
    }

    if let Some(RawFree { used: Some(used), limit: Some(limit) }) = k.free_model_daily_requests {
        if limit > 0.0 && limit.is_finite() && used.is_finite() {
            let (start, next) = period("daily", now).unwrap();
            windows.push(ApiWindow {
                id: "free_daily",
                label: "Modelos grátis · diário",
                used_pct: (used / limit * 100.0).clamp(0.0, 100.0),
                resets_at: Some(next),
                length: next - start,
            });
        }
    }

    let plan = k.usage_monthly.filter(|u| u.is_finite()).map(|u| format!("US$ {u:.2} no mês"));
    Ok(Usage { plan, windows })
}

pub async fn fetch_key(client: &reqwest::Client, url: &str, key: &str, now: DateTime<Utc>) -> Result<Usage, ApiError> {
    let resp = client
        .get(url)
        .bearer_auth(key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        401 | 403 => return Err(ApiError::Unauthorized),
        429 => return Err(ApiError::RateLimited),
        s if !(200..300).contains(&s) => return Err(ApiError::BadResponse(format!("HTTP {s}"))),
        _ => {}
    }
    let body = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;
    parse_key(&body, now).map_err(|e| ApiError::BadResponse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // quarta-feira, 14/10/2026 15:30 UTC
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 14, 15, 30, 0).unwrap()
    }

    const LIMITED: &str = r#"{"data":{"label":"sk-or-v1-abc...xyz","limit":50.0,"limit_remaining":38.0,"limit_reset":"monthly",
        "usage":120.5,"usage_daily":1.0,"usage_weekly":4.0,"usage_monthly":12.0,"is_free_tier":false,
        "free_model_daily_requests":{"used":10,"limit":50,"remaining":40}}}"#;

    #[test]
    fn limited_key_gives_the_limit_bar_and_the_free_models_bar() {
        let u = parse_key(LIMITED, now()).unwrap();
        assert_eq!(u.plan.as_deref(), Some("US$ 12.00 no mês"));
        let ids: Vec<_> = u.windows.iter().map(|w| w.id).collect();
        assert_eq!(ids, ["key_limit", "free_daily"]);
        assert_eq!(u.windows[0].used_pct, 24.0);
        assert_eq!(u.windows[0].resets_at, Some(Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap()));
        assert_eq!(u.windows[0].length, Duration::days(31));
        assert_eq!(u.windows[1].used_pct, 20.0);
        assert_eq!(u.windows[1].resets_at, Some(Utc.with_ymd_and_hms(2026, 10, 15, 0, 0, 0).unwrap()));
    }

    #[test]
    fn unlimited_key_has_no_bar_but_still_shows_the_monthly_spend() {
        let u = parse_key(r#"{"data":{"limit":null,"limit_remaining":null,"limit_reset":null,"usage_monthly":3.456}}"#, now()).unwrap();
        assert!(u.windows.is_empty());
        assert_eq!(u.plan.as_deref(), Some("US$ 3.46 no mês"));
    }

    #[test]
    fn reset_periods_follow_the_documented_utc_boundaries() {
        let d = period("daily", now()).unwrap();
        assert_eq!(d, (Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 10, 15, 0, 0, 0).unwrap()));
        // semana começa na segunda (12/10/2026)
        let w = period("weekly", now()).unwrap();
        assert_eq!(w, (Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 10, 19, 0, 0, 0).unwrap()));
        let dec = Utc.with_ymd_and_hms(2026, 12, 20, 8, 0, 0).unwrap();
        assert_eq!(period("monthly", dec).unwrap().1, Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap());
        assert_eq!(period("yearly", now()), None);
    }

    #[test]
    fn unknown_reset_kind_keeps_the_bar_without_a_reset_time() {
        let u = parse_key(r#"{"data":{"limit":10.0,"limit_remaining":5.0,"limit_reset":"decadal"}}"#, now()).unwrap();
        assert_eq!((u.windows[0].used_pct, u.windows[0].resets_at), (50.0, None));
    }

    #[test]
    fn nonsense_numbers_are_ignored_or_clamped() {
        let u = parse_key(r#"{"data":{"limit":0.0,"limit_remaining":0.0,"free_model_daily_requests":{"used":90,"limit":50}}}"#, now()).unwrap();
        assert_eq!(u.windows.len(), 1);
        assert_eq!(u.windows[0].used_pct, 100.0);
        assert!(parse_key("{}", now()).unwrap().windows.is_empty());
        assert!(parse_key("não é json", now()).is_err());
    }

    async fn server(code: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/key").match_header("authorization", "Bearer k").with_status(code).with_body(LIMITED).create_async().await;
        let url = format!("{}/key", s.url());
        (s, url)
    }

    #[tokio::test]
    async fn fetch_sends_the_key_as_bearer_and_parses() {
        let (_s, url) = server(200).await;
        let u = fetch_key(&reqwest::Client::new(), &url, "k", now()).await.unwrap();
        assert_eq!(u.windows.len(), 2);
    }

    #[tokio::test]
    async fn http_statuses_map_to_errors() {
        for (code, want) in [(401, ApiError::Unauthorized), (403, ApiError::Unauthorized), (429, ApiError::RateLimited)] {
            let (_s, url) = server(code).await;
            assert_eq!(fetch_key(&reqwest::Client::new(), &url, "k", now()).await.unwrap_err(), want, "{code}");
        }
        let (_s, url) = server(500).await;
        assert!(matches!(fetch_key(&reqwest::Client::new(), &url, "k", now()).await, Err(ApiError::BadResponse(_))));
        let r = fetch_key(&reqwest::Client::new(), "http://127.0.0.1:9/key", "k", now()).await;
        assert!(matches!(r, Err(ApiError::Network(_))));
    }
}
