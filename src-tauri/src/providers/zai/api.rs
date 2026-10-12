//! Cota do plano de código do z.ai (GLM): `GET <domínio>/api/monitor/usage/quota/limit`.
//! Formato e cabeçalho conferidos no plugin oficial `zai-org/zai-coding-plugins`
//! (`glm-plan-usage/.../query-usage.mjs`): `Authorization` leva o token puro, sem "Bearer", e
//! `data.limits[]` traz `type` (`TOKENS_LIMIT` = ciclo de 5 h, `TIME_LIMIT` = MCP no mês) e
//! `percentage`. `nextResetTime` não aparece nesse script, então é lido só se vier.

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::Deserialize;

pub const QUOTA_PATH: &str = "/api/monitor/usage/quota/limit";

#[derive(Deserialize)]
struct RawLimit {
    #[serde(rename = "type")]
    kind: Option<String>,
    percentage: Option<f64>,
    #[serde(rename = "nextResetTime")]
    next_reset_time: Option<i64>,
}

#[derive(Deserialize)]
struct RawData {
    limits: Option<Vec<RawLimit>>,
}

#[derive(Deserialize)]
struct RawResponse {
    data: Option<RawData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiWindow {
    pub id: String,
    pub label: String,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub length: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Unauthorized,
    RateLimited,
    Network(String),
    BadResponse(String),
}

/// Segundos ou milissegundos desde 1970 (o campo não é documentado; ms é sempre > 1e11).
fn reset_time(raw: Option<i64>) -> Option<DateTime<Utc>> {
    let t = raw.filter(|t| *t > 0)?;
    if t > 100_000_000_000 { Utc.timestamp_millis_opt(t).single() } else { Utc.timestamp_opt(t, 0).single() }
}

pub fn parse_quota(body: &str) -> Result<Vec<ApiWindow>, serde_json::Error> {
    let raw: RawResponse = serde_json::from_str(body)?;
    let limits = raw.data.and_then(|d| d.limits).unwrap_or_default();
    let mut token_cotas = 0;
    let mut out = Vec::new();
    for l in limits {
        let Some(pct) = l.percentage.filter(|p| p.is_finite()) else { continue };
        let used_pct = pct.clamp(0.0, 100.0);
        let resets_at = reset_time(l.next_reset_time);
        match l.kind.as_deref() {
            Some("TOKENS_LIMIT") => {
                token_cotas += 1;
                // Só a 1ª está confirmada como o ciclo de 5 h; qualquer outra cota de tokens
                // aparece com nome neutro em vez de adivinhar que é a semanal.
                let (id, label, length) = if token_cotas == 1 {
                    ("five_hour".to_string(), "Sessão (5h)".to_string(), Duration::hours(5))
                } else {
                    (format!("tokens_{token_cotas}"), "Tokens · outra cota".to_string(), Duration::days(7))
                };
                out.push(ApiWindow { id, label, used_pct, resets_at, length });
            }
            Some("TIME_LIMIT") => out.push(ApiWindow {
                id: "mcp_month".into(),
                label: "MCP · mensal".into(),
                used_pct,
                resets_at,
                length: Duration::days(30),
            }),
            _ => {}
        }
    }
    Ok(out)
}

pub async fn fetch_quota(client: &reqwest::Client, url: &str, token: &str) -> Result<Vec<ApiWindow>, ApiError> {
    let resp = client
        .get(url)
        .header("Authorization", token)
        .header("Accept-Language", "en-US,en")
        .header("Content-Type", "application/json")
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
    parse_quota(&body).map_err(|e| ApiError::BadResponse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = r#"{"code":200,"data":{"limits":[
        {"type":"TOKENS_LIMIT","unit":3,"number":5,"percentage":37.0,"nextResetTime":1790010000000},
        {"type":"TIME_LIMIT","unit":5,"number":1,"percentage":12.0,"currentValue":12,"usage":100,"usageDetails":[]}]}}"#;

    #[test]
    fn maps_the_two_documented_limit_types() {
        let ws = parse_quota(BODY).unwrap();
        assert_eq!(ws.len(), 2);
        assert_eq!((ws[0].id.as_str(), ws[0].label.as_str(), ws[0].used_pct), ("five_hour", "Sessão (5h)", 37.0));
        assert_eq!(ws[0].resets_at, Some(Utc.timestamp_millis_opt(1_790_010_000_000).unwrap()));
        assert_eq!(ws[0].length, Duration::hours(5));
        assert_eq!((ws[1].id.as_str(), ws[1].label.as_str(), ws[1].used_pct), ("mcp_month", "MCP · mensal", 12.0));
        assert_eq!(ws[1].resets_at, None);
    }

    #[test]
    fn a_second_token_quota_gets_a_neutral_name_instead_of_a_guess() {
        let body = r#"{"data":{"limits":[{"type":"TOKENS_LIMIT","percentage":10},{"type":"TOKENS_LIMIT","percentage":55}]}}"#;
        let ws = parse_quota(body).unwrap();
        assert_eq!((ws[0].id.as_str(), ws[1].id.as_str()), ("five_hour", "tokens_2"));
        assert_eq!(ws[1].label, "Tokens · outra cota");
    }

    #[test]
    fn reset_time_accepts_seconds_or_milliseconds() {
        assert_eq!(reset_time(Some(1_790_010_000)), Some(Utc.timestamp_opt(1_790_010_000, 0).unwrap()));
        assert_eq!(reset_time(Some(1_790_010_000_000)), Some(Utc.timestamp_opt(1_790_010_000, 0).unwrap()));
        assert_eq!(reset_time(Some(0)), None);
        assert_eq!(reset_time(None), None);
    }

    #[test]
    fn unknown_types_missing_or_odd_numbers_are_ignored_and_clamped() {
        let body = r#"{"data":{"limits":[{"type":"OTHER","percentage":5},{"type":"TOKENS_LIMIT"},{"type":"TIME_LIMIT","percentage":250}]}}"#;
        let ws = parse_quota(body).unwrap();
        assert_eq!(ws.len(), 1);
        assert_eq!((ws[0].id.as_str(), ws[0].used_pct), ("mcp_month", 100.0));
        assert!(parse_quota("{}").unwrap().is_empty());
        assert!(parse_quota(r#"{"data":{}}"#).unwrap().is_empty());
        assert!(parse_quota("não é json").is_err());
    }

    async fn server(code: usize) -> (mockito::ServerGuard, String) {
        let mut s = mockito::Server::new_async().await;
        // o token vai puro, sem "Bearer"
        s.mock("GET", QUOTA_PATH).match_header("authorization", "tok").with_status(code).with_body(BODY).create_async().await;
        let url = format!("{}{}", s.url(), QUOTA_PATH);
        (s, url)
    }

    #[tokio::test]
    async fn fetch_sends_the_raw_token_and_parses() {
        let (_s, url) = server(200).await;
        assert_eq!(fetch_quota(&reqwest::Client::new(), &url, "tok").await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn http_statuses_map_to_errors() {
        for (code, want) in [(401, ApiError::Unauthorized), (403, ApiError::Unauthorized), (429, ApiError::RateLimited)] {
            let (_s, url) = server(code).await;
            assert_eq!(fetch_quota(&reqwest::Client::new(), &url, "tok").await.unwrap_err(), want, "{code}");
        }
        let (_s, url) = server(500).await;
        assert!(matches!(fetch_quota(&reqwest::Client::new(), &url, "tok").await, Err(ApiError::BadResponse(_))));
        let r = fetch_quota(&reqwest::Client::new(), "http://127.0.0.1:9/x", "tok").await;
        assert!(matches!(r, Err(ApiError::Network(_))));
    }
}
