use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOauth {
    access_token: String,
    expires_at: i64,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFile {
    claude_ai_oauth: Option<RawOauth>,
}

#[derive(Clone, PartialEq)]
pub struct Creds {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub plan: Option<String>,
}

impl fmt::Debug for Creds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creds")
            .field("access_token", &"<oculto>")
            .field("expires_at", &self.expires_at)
            .field("plan", &self.plan)
            .finish()
    }
}

impl Creds {
    /// Considera vencido 1 minuto antes, pra não mandar token no limite.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now + Duration::seconds(60) >= self.expires_at
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CredsError {
    /// Sem arquivo ou sem login do Claude.ai nele.
    Missing,
    /// Arquivo existe mas não deu pra ler/parsear (ex.: Claude Code reescrevendo agora).
    Unreadable(String),
}

pub fn claude_dir() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".claude"))
}

/// Somente leitura: o Pacer nunca renova nem escreve neste arquivo.
pub fn read(claude_dir: &Path) -> Result<Creds, CredsError> {
    let path = claude_dir.join(".credentials.json");
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(CredsError::Missing),
        Err(e) => return Err(CredsError::Unreadable(e.to_string())),
    };
    let file: RawFile = serde_json::from_str(&raw).map_err(|e| CredsError::Unreadable(e.to_string()))?;
    let o = file.claude_ai_oauth.ok_or(CredsError::Missing)?;
    let expires_at = DateTime::from_timestamp_millis(o.expires_at)
        .ok_or_else(|| CredsError::Unreadable("expiresAt inválido".into()))?;
    Ok(Creds {
        access_token: o.access_token,
        expires_at,
        plan: plan_label(o.subscription_type.as_deref(), o.rate_limit_tier.as_deref()),
    })
}

pub fn plan_label(sub: Option<&str>, tier: Option<&str>) -> Option<String> {
    let tier = tier.unwrap_or("").to_ascii_lowercase();
    if tier.contains("max_20x") {
        return Some("Max 20x".into());
    }
    if tier.contains("max_5x") {
        return Some("Max 5x".into());
    }
    let label = match sub.map(str::to_ascii_lowercase).as_deref() {
        Some("max") => "Max",
        Some("pro") => "Pro",
        Some("team") => "Team",
        Some("enterprise") => "Enterprise",
        Some("free") => "Free",
        _ => return None,
    };
    Some(label.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use std::fs;

    fn write(dir: &Path, body: &str) {
        fs::write(dir.join(".credentials.json"), body).unwrap();
    }

    #[test]
    fn missing_file_is_missing() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(read(d.path()), Err(CredsError::Missing));
    }

    #[test]
    fn file_without_claude_oauth_is_missing() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"mcpOAuth":{}}"#);
        assert_eq!(read(d.path()), Err(CredsError::Missing));
    }

    #[test]
    fn half_written_file_is_unreadable_not_missing() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"claudeAiOauth":{"accessTo"#);
        assert!(matches!(read(d.path()), Err(CredsError::Unreadable(_))));
    }

    #[test]
    fn reads_token_expiry_and_plan() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat-x","refreshToken":"r","expiresAt":1791043200000,"scopes":[],"subscriptionType":"pro","rateLimitTier":"default_claude_ai"}}"#);
        let c = read(d.path()).unwrap();
        assert_eq!(c.access_token, "sk-ant-oat-x");
        assert_eq!(c.expires_at, Utc.timestamp_millis_opt(1791043200000).unwrap());
        assert_eq!(c.plan.as_deref(), Some("Pro"));
    }

    #[test]
    fn debug_does_not_leak_token() {
        let c = Creds { access_token: "segredo".into(), expires_at: Utc::now(), plan: None };
        assert!(!format!("{c:?}").contains("segredo"));
    }

    #[test]
    fn expiry_has_one_minute_margin() {
        let exp = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
        let c = Creds { access_token: "t".into(), expires_at: exp, plan: None };
        assert!(!c.is_expired(exp - Duration::seconds(61)));
        assert!(c.is_expired(exp - Duration::seconds(59)));
    }

    #[test]
    fn plan_labels() {
        assert_eq!(plan_label(Some("max"), Some("default_claude_max_20x")).as_deref(), Some("Max 20x"));
        assert_eq!(plan_label(Some("max"), Some("default_claude_max_5x")).as_deref(), Some("Max 5x"));
        assert_eq!(plan_label(Some("max"), None).as_deref(), Some("Max"));
        assert_eq!(plan_label(Some("pro"), Some("default_claude_ai")).as_deref(), Some("Pro"));
        assert_eq!(plan_label(Some("team"), None).as_deref(), Some("Team"));
        assert_eq!(plan_label(None, None), None);
    }
}
