use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

pub const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA: &str = "oauth-2025-04-20";

#[derive(Deserialize)]
struct RawWindow {
    utilization: Option<f64>,
    resets_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct RawUsage {
    five_hour: Option<RawWindow>,
    seven_day: Option<RawWindow>,
    seven_day_opus: Option<RawWindow>,
    seven_day_sonnet: Option<RawWindow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiWindow {
    pub id: &'static str,
    pub label: &'static str,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub length: Duration,
}

/// Janelas `null`, sem `utilization` ou desconhecidas são ignoradas.
pub fn parse_usage(body: &str) -> Result<Vec<ApiWindow>, serde_json::Error> {
    let raw: RawUsage = serde_json::from_str(body)?;
    let specs = [
        ("five_hour", "Sessão (5h)", Duration::hours(5), raw.five_hour),
        ("seven_day", "Semanal", Duration::days(7), raw.seven_day),
        ("seven_day_opus", "Opus · semanal", Duration::days(7), raw.seven_day_opus),
        ("seven_day_sonnet", "Sonnet · semanal", Duration::days(7), raw.seven_day_sonnet),
    ];
    Ok(specs
        .into_iter()
        .filter_map(|(id, label, length, w)| {
            let w = w?;
            let used = w.utilization?;
            Some(ApiWindow { id, label, used_pct: used.clamp(0.0, 100.0), resets_at: w.resets_at, length })
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Unauthorized,
    RateLimited,
    Network(String),
    BadResponse(String),
}

pub fn user_agent(cli_version: &str) -> String {
    format!("claude-cli/{cli_version} (external, cli)")
}

pub async fn fetch_usage(client: &reqwest::Client, url: &str, token: &str, ua: &str) -> Result<Vec<ApiWindow>, ApiError> {
    let resp = client
        .get(url)
        .bearer_auth(token)
        .header("anthropic-beta", BETA)
        .header("User-Agent", ua)
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
    parse_usage(&body).map_err(|e| ApiError::BadResponse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const FULL: &str = include_str!("../../../tests/fixtures/usage_full.json");
    const MINIMAL: &str = include_str!("../../../tests/fixtures/usage_minimal.json");

    #[test]
    fn parses_full_response_ignoring_nulls_and_unknown_fields() {
        let ws = parse_usage(FULL).unwrap();
        let ids: Vec<_> = ws.iter().map(|w| w.id).collect();
        assert_eq!(ids, ["five_hour", "seven_day", "seven_day_sonnet"]);
        assert_eq!(ws[0].label, "Sessão (5h)");
        assert_eq!(ws[0].used_pct, 62.0);
        assert_eq!(ws[0].length, Duration::hours(5));
        assert_eq!(ws[0].resets_at, Some(Utc.with_ymd_and_hms(2026, 5, 23, 13, 30, 0).unwrap()));
        assert_eq!(ws[2].label, "Sonnet · semanal");
        assert_eq!(ws[2].resets_at, Some(Utc.with_ymd_and_hms(2026, 5, 23, 14, 24, 0).unwrap()));
    }

    #[test]
    fn parses_minimal_response_with_null_reset() {
        let ws = parse_usage(MINIMAL).unwrap();
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[1].label, "Semanal");
        assert_eq!(ws[1].resets_at, None);
    }

    #[test]
    fn window_without_utilization_is_skipped_and_values_clamped() {
        let ws = parse_usage(r#"{"five_hour":{"resets_at":null},"seven_day":{"utilization":130.0}}"#).unwrap();
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].used_pct, 100.0);
    }

    #[test]
    fn user_agent_format() {
        assert_eq!(user_agent("2.1.288"), "claude-cli/2.1.288 (external, cli)");
    }

    async fn serve(status: usize, body: &str) -> (mockito::ServerGuard, mockito::Mock) {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/usage")
            .match_header("authorization", "Bearer tok")
            .match_header("anthropic-beta", "oauth-2025-04-20")
            .match_header("user-agent", "ua-teste")
            .with_status(status)
            .with_body(body)
            .create_async()
            .await;
        (server, mock)
    }

    #[tokio::test]
    async fn fetch_ok() {
        let (server, mock) = serve(200, MINIMAL).await;
        let ws = fetch_usage(&reqwest::Client::new(), &format!("{}/usage", server.url()), "tok", "ua-teste").await.unwrap();
        mock.assert_async().await;
        assert_eq!(ws.len(), 2);
    }

    #[tokio::test]
    async fn fetch_maps_status_codes() {
        let client = reqwest::Client::new();
        for (status, expected) in [
            (401, ApiError::Unauthorized),
            (403, ApiError::Unauthorized),
            (429, ApiError::RateLimited),
            (500, ApiError::BadResponse("HTTP 500".into())),
        ] {
            let (server, _m) = serve(status, "{}").await;
            let err = fetch_usage(&client, &format!("{}/usage", server.url()), "tok", "ua-teste").await.unwrap_err();
            assert_eq!(err, expected, "status {status}");
        }
    }

    #[tokio::test]
    async fn fetch_network_error() {
        let err = fetch_usage(&reqwest::Client::new(), "http://127.0.0.1:9/usage", "tok", "ua").await.unwrap_err();
        assert!(matches!(err, ApiError::Network(_)));
    }
}
