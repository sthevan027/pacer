use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pace {
    pub projected_pct: f64,
    pub limit_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub id: String,
    pub label: String,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub pace: Option<Pace>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayActivity {
    pub date: NaiveDate,
    pub tokens: u64,
    pub messages: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Notice {
    NoCredentials,
    TokenExpired,
    RateLimited,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Ok,
    Warn,
    Critical,
}

/// Só o uso decide a cor (a previsão não): azul < 70%, laranja→vermelho 70–90%, vermelho ≥ 90%.
pub fn severity(used_pct: f64) -> Severity {
    if used_pct >= 90.0 {
        Severity::Critical
    } else if used_pct >= 70.0 {
        Severity::Warn
    } else {
        Severity::Ok
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub provider: String,
    pub name: String,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    /// 30 itens, do mais antigo até hoje.
    pub activity: Vec<DayActivity>,
    pub today_tokens: u64,
    /// Última busca remota bem-sucedida (None = nunca / só logs).
    pub fetched_at: Option<DateTime<Utc>>,
    /// Preenchido pelo scheduler.
    pub next_refresh_at: Option<DateTime<Utc>>,
    pub stale: bool,
    pub notice: Option<Notice>,
}

impl Snapshot {
    pub fn max_severity(&self) -> Severity {
        self.windows
            .iter()
            .map(|w| severity(w.used_pct))
            .max()
            .unwrap_or(Severity::Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn pace(p: f64) -> Pace {
        Pace { projected_pct: p, limit_at: None }
    }

    #[test]
    fn severity_follows_usage_only() {
        // azul até 70%, laranja→vermelho de 70 a 90%, vermelho a partir de 90%
        assert_eq!(severity(0.0), Severity::Ok);
        assert_eq!(severity(69.9), Severity::Ok);
        assert_eq!(severity(70.0), Severity::Warn);
        assert_eq!(severity(89.9), Severity::Warn);
        assert_eq!(severity(90.0), Severity::Critical);
        assert_eq!(severity(100.0), Severity::Critical);
    }

    #[test]
    fn high_projection_with_low_usage_is_not_alarming() {
        // a previsão estourando não pinta a barra de vermelho: 22% usado continua azul
        let w = UsageWindow {
            id: "five_hour".into(),
            label: "Sessão (5h)".into(),
            used_pct: 22.0,
            resets_at: None,
            pace: Some(pace(180.0)),
        };
        let s = Snapshot {
            provider: "claude".into(), name: "Claude".into(), plan: None,
            windows: vec![w], activity: vec![], today_tokens: 0,
            fetched_at: None, next_refresh_at: None, stale: false, notice: None,
        };
        assert_eq!(s.max_severity(), Severity::Ok);
    }

    #[test]
    fn serializes_camel_case() {
        let s = Snapshot {
            provider: "claude".into(),
            name: "Claude".into(),
            plan: Some("Pro".into()),
            windows: vec![UsageWindow {
                id: "five_hour".into(),
                label: "Sessão (5h)".into(),
                used_pct: 38.0,
                resets_at: Some(Utc.with_ymd_and_hms(2026, 10, 3, 17, 0, 0).unwrap()),
                pace: Some(pace(61.0)),
            }],
            activity: vec![DayActivity { date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(), tokens: 10, messages: 2 }],
            today_tokens: 10,
            fetched_at: None,
            next_refresh_at: None,
            stale: false,
            notice: Some(Notice::NoCredentials),
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["todayTokens"], 10);
        assert_eq!(v["windows"][0]["usedPct"], 38.0);
        assert_eq!(v["windows"][0]["resetsAt"], "2026-10-03T17:00:00Z");
        assert_eq!(v["windows"][0]["pace"]["projectedPct"], 61.0);
        assert_eq!(v["activity"][0]["date"], "2026-10-03");
        assert_eq!(v["notice"], "noCredentials");
        assert_eq!(serde_json::to_value(Severity::Critical).unwrap(), "critical");
    }

    #[test]
    fn max_severity_picks_worst_window() {
        let w = |p: f64| UsageWindow { id: "x".into(), label: "x".into(), used_pct: p, resets_at: None, pace: None };
        let mut s = Snapshot {
            provider: "claude".into(), name: "Claude".into(), plan: None,
            windows: vec![w(10.0), w(80.0)], activity: vec![], today_tokens: 0,
            fetched_at: None, next_refresh_at: None, stale: false, notice: None,
        };
        assert_eq!(s.max_severity(), Severity::Warn);
        s.windows.clear();
        assert_eq!(s.max_severity(), Severity::Ok);
    }
}
